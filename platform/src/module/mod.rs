use osc::module::ModuleInfo;

mod client;

pub use client::{Library, Session};

trait Client {
    type Handle;
    type Context;

    fn init(&self, context: Self::Context) -> Result<(), anyhow::Error>;

    fn info(&self) -> ModuleInfo;

    fn set<T>(&self, handle: Self::Handle, payload: &T) -> Result<(), anyhow::Error>
    where
        T: serde::Serialize;

    fn get<T>(&self, session: Self::Handle) -> Result<T, anyhow::Error>
    where
        T: serde::de::DeserializeOwned;
}
