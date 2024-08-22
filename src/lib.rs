mod config;
mod error;
mod models;
mod module;

pub mod handlers;
pub mod routes;

use std::sync::{Arc, Mutex};

pub use error::{Error, Result};
use module::ModuleManager;

pub(crate) const PLATFORM_CLIENT: &str = "osc-platform-rs";

pub type Platform = Arc<Mutex<ModuleManager>>;

pub fn init() -> Result<Platform> {
    log::info!("{}", PLATFORM_CLIENT);

    let platform = ModuleManager::new("/usr/lib/osconfig")?;
    Ok(Arc::new(Mutex::new(platform)))
}
