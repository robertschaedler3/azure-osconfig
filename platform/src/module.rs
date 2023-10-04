use std::collections::HashMap;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::action;

pub struct Module {
    properties: HashMap<String, Property>,
}

impl Module {
    pub fn new(manifest: Manifest) -> Result<(String, Self)> {
        let name = manifest.name;
        let properties = manifest
            .properties
            .into_iter()
            .map(|property| {
                let name = property.name.clone();
                (name, property)
            })
            .collect();

        let module = Self { properties };

        Ok((name, module))
    }

    pub fn resolve(&self, property: &str) -> Result<Value> {
        self.properties
            .get(property)
            .context("Failed to find property")?
            .read
            .clone()
            .context("Property is not readable")?
            .resolve()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Manifest {
    name: String,
    properties: Vec<Property>, // TODO: other optional metadata (version, manufacturer, license, etc.)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Property {
    name: String,
    schema: Schema,

    // TODO: must have at least read or write

    read: Option<Read>,
    // write: Option<action::Write>
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
enum Schema {
    Number,
    Bool,
    String,
}

// trait Action {
//     fn resolve(&self) -> Result<Value>;
// }

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Read {
    #[serde(flatten)]
    action: ReadAction,

    // TODO: fields shared by all read actions - default...
}

// impl Action for Read {
impl Read {
    fn resolve(&self) -> Result<Value> {
        match &self.action {
            ReadAction::FileContent { path } => action::read_file_content(path),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "action")]
pub enum ReadAction {
    #[serde(rename = "read-file-content")]
    FileContent {
        path: String,

        // REVIEW: optional fields - lock, timeout, retry, regex, filter
    },
}
