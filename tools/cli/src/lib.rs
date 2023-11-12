use std::path::PathBuf;

use module::{DefaultClient, Client};
use osc::module::Meta;

pub mod module;

pub(crate) const PLATFORM_CLIENT: &str = "osc_cli";

pub struct Module<T: Client = DefaultClient> {
    pub path: PathBuf,
    pub meta: Meta,
    client: T,
}

impl Module {
    pub fn new(path: PathBuf) -> anyhow::Result<Self> {
        let client =  DefaultClient::load(&path)?;
        let meta = client.meta()?;
        Ok(Self { path, client, meta })
    }

    pub fn get(&self, component: &str, object: &str) -> anyhow::Result<serde_json::Value> {
        self.client.get(component, object)
    }

    pub fn set(&self, component: &str, object: &str, payload: &serde_json::Value) -> anyhow::Result<()> {
        self.client.set(component, object, payload)
    }

    pub fn compnents(&self) -> &[String] {
        &self.meta.components
    }
}
