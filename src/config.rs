use std::collections::HashMap;

use serde::de::{self, Deserializer, Unexpected};
use serde::Deserialize;
use serde_repr::Deserialize_repr;

const CONFIG_PATH: &str = "/etc/osconfig/osconfig.json";

/// Load the configuration from `/etc/osconfig/osconfig.json` or provides the default configuration if the file does not exist.
pub fn load() -> Config {
    std::fs::read_to_string(CONFIG_PATH)
        .map(|s| serde_json::from_str::<Config>(&s).unwrap_or_default())
        .unwrap_or_default()
}

#[derive(Default, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Config {
    /// Command logging means that OSConfig will log all input and output from system commands executed by Agent, Platform and Modules.
    ///
    /// _Generally it is not recommended to run OSConfig with command logging enabled._
    #[serde(deserialize_with = "bool_from_int")]
    pub _command_logging: bool,

    /// Full logging means that OSConfig will log all input and output from and to IoT Hub, AIS, RC/DC, GitOps DC, etc.
    ///
    /// _Generally it is not recommended to run OSConfig with full logging enabled._
    #[serde(deserialize_with = "bool_from_int")]
    pub _full_logging: bool,

    pub reported: Reported,
    // #[serde(deserialize_with = "bool_from_int")]
    // pub git_management: bool,

    // pub repository_url: Option<String>,
    // pub repository_branch: Option<String>,

    // #[serde(deserialize_with = "bool_from_int")]
    // pub local_management: bool,

    // pub model_version: u32,
    // pub iothub_protocol: IotHubProtocol,

    // #[serde(rename = "ReportingIntervalSeconds")]
    // pub reporting_interval: u32,
}

/// The networking protocol that OSConfig uses to connect to the Azure IoTHub.
#[derive(Deserialize_repr)]
#[repr(u8)]
pub enum IotHubProtocol {
    /// Decided by OSConfig (currently MQTT).
    Auto = 0,
    Mqtt = 1,
    MqttWS = 2,
}

#[derive(Default)]
pub struct Reported(pub HashMap<String, Vec<String>>);

#[derive(Deserialize)]
struct Pair {
    #[serde(rename = "ComponentName")]
    pub component: String,

    #[serde(rename = "ObjectName")]
    pub object: String,
}

/// Aggregate the Vec of Reported {component, objects} into a Map of component -> Vec<object>
impl<'de> Deserialize<'de> for Reported {
    fn deserialize<D>(deserializer: D) -> Result<Reported, D::Error>
    where
        D: Deserializer<'de>,
    {
        let pairs = Vec::<Pair>::deserialize(deserializer)?;
        let reported = pairs
            .into_iter()
            .fold(HashMap::new(), |mut acc, pair| {
                let objects = acc.entry(pair.component).or_insert_with(Vec::new);
                objects.push(pair.object);
                acc
            })
            .into();
        Ok(Reported(reported))
    }
}

fn bool_from_int<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    match u8::deserialize(deserializer)? {
        0 => Ok(false),
        1 => Ok(true),
        other => Err(de::Error::invalid_value(
            Unexpected::Unsigned(other as u64),
            &"zero or one",
        )),
    }
}
