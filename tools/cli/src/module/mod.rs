use std::path::Path;

use osc::module::Meta;

mod sharedlib;

pub use sharedlib::SharedLibClient as DefaultClient;

pub type Payload = serde_json::Value;

pub trait Client: Sized {
    fn load<P: AsRef<Path>>(path: P) -> anyhow::Result<Self>;
    fn meta(&self) -> anyhow::Result<Meta>;
    fn get(&self, component: &str, object: &str) -> anyhow::Result<Payload>;
    fn set(&self, component: &str, object: &str, payload: &Payload) -> anyhow::Result<()>;
}
