use std::{
    fs::File,
    io::{BufReader, Write},
    path::Path,
};

use anyhow::{anyhow, Result};
use format_serde_error::SerdeError;
use serde::Deserialize;

use osc::module::schema;

use super::fixture::Fixture;

#[derive(Deserialize)]
pub struct Definition {
    #[serde(default)]
    pub config: Config,
    pub modules: Vec<String>,
    pub setup: Option<Script>,
    pub teardown: Option<Script>,
    pub steps: Vec<Step>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Config {
    pub client_name: String,
    pub max_payload_size: u32,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Script(String);

#[derive(Deserialize)]
pub struct Step {
    #[serde(flatten)]
    pub action: Action,

    #[serde(default)]
    pub assert: Assertion,

    #[serde(flatten)]
    #[serde(default)]
    pub options: Options,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Action {
    Get {
        component: String,
        object: String,
    },
    Set {
        component: String,
        object: String,

        #[serde(flatten)]
        value: Value,

        size: Option<i32>,
    },
}

#[derive(Clone, Deserialize)]
pub struct Assertion {
    #[serde(flatten)]
    value: Option<Value>,

    // TODO: size should only be allowed with Valie::Json
    size: Option<i32>,

    #[serde(default)]
    status: Status,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(untagged)]
#[serde(rename_all = "lowercase")]
enum Status {
    // FIXME: success/failure not working
    Success,
    Failure,
    Exit(i32),
}

#[derive(Default, Debug, Deserialize)]
pub struct Options {
    /// Delay in milliseconds before executing the step.
    pub delay: Option<u64>,

    /// Skip this test step.
    pub skip: Option<bool>,
}

#[derive(Clone, Debug, Deserialize)]
pub enum Value {
    #[serde(rename = "payload")]
    Payload(schema::Value),

    #[serde(rename = "json")]
    // TODO: validate this JSON string (https://github.com/serde-rs/serde/issues/939#issuecomment-939514114)
    Json(Json),
}

#[derive(Clone, Debug, Deserialize)]
#[serde(try_from = "String")] // Tell serde to deserialize data into a String and then try to convert it into JSON
pub struct Json(schema::Value);

impl TryFrom<String> for Json {
    type Error = String;

    fn try_from(value: String) -> std::result::Result<Self, Self::Error> {
        let value = serde_json::from_str(&value)
            .map_err(|err| SerdeError::new(value.to_string(), err))
            .unwrap();
        Ok(Self(value))
    }
}

impl Definition {
    pub fn from_file(path: &Path) -> Result<Self> {
        let file = File::open(path).map_err(|err| anyhow!("failed to open file: {}", err))?;
        let reader = BufReader::new(file);
        let definition: Self = serde_yaml::from_reader(reader).map_err(|err| {
            let err = SerdeError::new(path.to_string_lossy().to_string(), err);
            anyhow!("failed to parse YAML: {}", err)
        })?;
        Ok(definition)
    }

    pub fn into_parts(&self) -> (Option<Script>, Option<Script>, Fixture) {
        let setup = self.setup.clone();
        let teardown = self.teardown.clone();
        let fixture = self.into();
        (setup, teardown, fixture)
    }
}

impl Script {
    pub fn execute(&self) -> Result<String> {
        let script = &self.0;

        // Write the script to a temporary file
        let mut file = tempfile::NamedTempFile::new()?;
        file.write_all(script.as_bytes())?;

        let output = std::process::Command::new("bash")
            .arg(file.path())
            .output()?;

        if !output.status.success() {
            return Err(anyhow!(
                "script failed ({}): {}",
                output.status,
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
}

impl From<&Value> for (String, i32) {
    fn from(value: &Value) -> Self {
        match value {
            Value::Payload(payload) => {
                let payload = serde_json::to_string(&payload).unwrap();
                let size = payload.len();
                (payload, size as i32)
            }
            Value::Json(json) => {
                let payload = serde_json::to_string(&json.0).unwrap();
                let size = payload.len();
                (payload, size as i32)
            }
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            // TODO: use "official" client name
            client_name: "osc".to_string(),
            max_payload_size: 0,
        }
    }
}

impl Default for Assertion {
    fn default() -> Self {
        Self {
            value: None,
            size: None,
            status: Status::Success,
        }
    }
}

impl Default for Status {
    fn default() -> Self {
        Status::Success
    }
}
