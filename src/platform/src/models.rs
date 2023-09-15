use std::collections::HashMap;

use serde::Deserialize;

use crate::module::Payload;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct OpenBody {
    pub client_name: String,
    pub max_payload_size_bytes: u32,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct CloseBody {
    pub client_session: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct GetBody {
    pub client_session: String,
    #[serde(rename = "ComponentName")]
    pub component: String,
    #[serde(rename = "ObjectName")]
    pub object: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct SetBody {
    #[serde(rename = "ComponentName")]
    pub component: String,
    #[serde(rename = "ObjectName")]
    pub object: String,
    pub payload: Payload,
    pub client_session: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct GetReportedBody {
    pub client_session: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct SetDesiredBody {
    pub payload: HashMap<String, HashMap<String, Payload>>,
    pub client_session: String,
}