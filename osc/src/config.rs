// TODO: use something like "confy" to load this configuration from the correct file

use serde::de::{self, Deserializer, Unexpected};
use serde::{Deserialize, Serialize, Serializer};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Config {
    // #[serde(serialize_with = "bool_to_int", deserialize_with = "bool_from_int")]
    command_logging: bool,

    #[serde(serialize_with = "bool_to_int")]
    #[serde(deserialize_with = "bool_from_int")]
    pub full_logging: bool,

    // local_management: bool,
    model_version: u32,
    // iothub_protocol: ???,
    reported: Vec<Reported>,

    #[serde(rename = "ReportingIntervalSeconds")]
    reporting_interval: u32,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Reported {
    component_name: String,
    object_name: String,
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

fn bool_to_int<S>(boolean: &bool, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_u8(*boolean as u8)
}