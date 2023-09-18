use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

mod config;
mod error;
mod models;
mod module;

pub mod handlers;
pub mod routes;

use config::Config;
use error::Error;
use module::{Client, Payload};

pub const PLATFORM_CLIENT: &str = env!("PLATFORM_CLIENT");
pub const MODULE_BIN_PATH: &str = "/usr/lib/osconfig";

pub type Data = Arc<Mutex<Platform>>;

pub struct Platform {
    modules: Vec<Box<Module>>,
    resources: HashMap<String>,
    config: Config,
}

struct Module<T: Client = DefaultClient> {
    path: PathBuf,
    client: Option<T>
}

pub fn init() -> Result<Arc<Mutex<Platform>>, Error> {
    log::info!("{}", PLATFORM_CLIENT);

    let platform = Platform::new("/usr/lib/osconfig")?;
    Ok(Arc::new(Mutex::new(platform)))
}

// impl<T: Provider> Platform<T> {
//     pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
//         let config = config::load()?;
//         Ok(Self {
//             modules: HashMap::new(),
//             config,
//         })
//     }

//     /// Initialize the platform by loading modules from `/usr/lib/osconfig`.
//     /// Modules are loaded in alphabetical order and the module with the greatest version number is kept.
//     fn load(&mut self) -> Result<(), Error> {
//         let mut paths: Vec<PathBuf> = std::fs::read_dir(&self.path)?
//             .map(|entry| entry.unwrap().path())
//             .filter(|path| path.extension().unwrap_or_default() == "so")
//             .collect();

//         paths.sort();

//         let failed = paths
//             .iter()
//             .filter_map(|path| {
//                 let name = path.file_stem()?.to_str()?.to_string();
//                 match T::init(path) {
//                     Ok(module) => {
//                         // Check if a module with the same name has already been loaded,
//                         // keep the one with the greatest version number.
//                         if let Some(existing) = self.modules.get(&name) {
//                             if module.meta().version > existing.meta().version {
//                                 self.modules.insert(name.clone(), module);
//                             }
//                         } else {
//                             self.modules.insert(name.clone(), module);
//                         }
//                         None
//                     }
//                     Err(err) => {
//                         log::error!("{}", err);
//                         Some(name)
//                     }
//                 }
//             })
//             .collect::<Vec<_>>();

//         if !failed.is_empty() {
//             log::error!("Failed to load modules: [{}]", failed.join(", "));
//         }

//         Ok(())
//     }

//     pub fn reload(&mut self) -> Result<(), Error> {
//         self.modules.clear();
//         self.load()
//     }

//     fn find_module(&self, component: &str) -> Result<&T, Error> {
//         self.modules
//             .get(component)
//             .ok_or_else(|| Error::ComponentNotFound(component.to_string()))
//     }

//     pub fn get(&self, component: &str, object: &str) -> Result<Payload, Error> {
//         let module = self.find_module(component)?;
//         let client = module.client().get_or_insert(T::Client::load(&module.path())?);
//         client.get(component, object)
//     }

//     pub fn set(&self, component: &str, object: &str, payload: &Payload) -> Result<(), Error> {
//         let module = self.find_module(component)?;
//         let client = module.client().get_or_insert(T::Client::load(&module.path())?);
//         client.set(component, object, payload)
//     }
// }

// #[cfg(test)]
// mod tests {
//     use crate::module::Payload;

//     use super::*;

//     #[derive(Clone)]
//     struct TestClient;

//     impl Client for TestClient {
//         fn load<P: AsRef<Path>>(_path: P) -> Result<Self, Error> {
//             Ok(Self)
//         }

//         fn meta(&self) -> Result<Metadata, Error> {
//             Ok(Metadata {
//                 name: "test".to_string(),
//                 description: None,
//                 manufacturer: None,
//                 version: "1.2.3.4".into(),
//                 components: vec!["test".to_string()],
//                 lifetime: Lifetime::Short,
//                 user_account: module::UserAccount::Root,
//             })
//         }

//         fn get(&self, _component: &str, _object: &str) -> Result<Payload, Error> {
//             Ok(Payload::Null)
//         }

//         fn set(&self, _component: &str, _object: &str, _payload: &Payload) -> Result<(), Error> {
//             Ok(())
//         }
//     }

//     #[test]
//     fn test_module() {
//         let module = Module::<TestClient>::init("/path/to/module.so").unwrap();

//         assert_eq!(module.meta.name, "test");
//         assert_eq!(module.meta.version, "1.2.3.4".into());
//         assert_eq!(module.meta.lifetime, Lifetime::Short);
//         assert_eq!(module.meta.user_account, module::UserAccount::Root);

//         assert!(module.get("test", "test").is_ok());
//         assert!(module.set("test", "test", &Payload::Null).is_ok());
//     }
// }
