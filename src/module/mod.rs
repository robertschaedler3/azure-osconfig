use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use adapter::{Adapter, ModuleAdapter};
use meta::{Lifetime, Metadata};

use crate::{
    config::{self, Config}, Error, Result
};

mod adapter;
mod bindings;
mod sharedlib;

pub mod meta;

pub type Payload = serde_json::Value;

pub struct Module<T: Adapter = ModuleAdapter> {
    path: PathBuf,
    client: Option<T>,
    meta: Metadata,
}

pub struct ModuleManager {
    path: PathBuf,
    modules: HashMap<String, Module>,
    pub(crate) config: Config,
}

impl ModuleManager {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let config = config::load();
        Ok(Self {
            path: path.as_ref().to_path_buf(),
            modules: HashMap::new(),
            config,
        })
    }

    /// Initialize the platform by loading modules from `/usr/lib/osconfig`.
    /// Modules are loaded in alphabetical order and the module with the greatest version number is kept.
    fn load(&mut self) -> Result<()> {
        let dir = match std::fs::read_dir(&self.path) {
            Ok(dir) => dir,
            Err(err) => {
                // Return early if the directory does not exist to keep the platform running.
                log::error!("Failed to load modules directory: {}", err);
                return Ok(());
            }
        };

        let mut paths: Vec<PathBuf> = dir
            .map(|entry| entry.unwrap().path())
            .filter(|path| path.extension().unwrap_or_default() == "so")
            .collect();

        paths.sort();

        let failed = paths
            .iter()
            .filter_map(|path| {
                let name = path.file_stem().unwrap().to_str().unwrap().to_string();
                match Module::init(path) {
                    Ok(module) => {
                        // Check if a module with the same name has already been loaded,
                        // keep the one with the greatest version number.
                        if let Some(existing) = self.modules.get(&name) {
                            if module.meta.version > existing.meta.version {
                                self.modules.insert(name.clone(), module);
                            }
                        } else {
                            self.modules.insert(name.clone(), module);
                        }
                        None
                    }
                    Err(err) => {
                        log::error!("{}", err);
                        Some(name)
                    }
                }
            })
            .collect::<Vec<_>>();

        if !failed.is_empty() {
            log::error!("Failed to load modules: [{}]", failed.join(", "));
        }

        Ok(())
    }

    pub fn reload(&mut self) -> Result<()> {
        self.modules.clear();
        self.load()
    }

    pub fn get(&self, component: &str, object: &str) -> Result<Payload> {
        log::debug!("ModuleManager::get({}, {})", component, object);

        let module = self
            .modules
            .values()
            .find(|module| module.meta.components.contains(&component.to_string()))
            .ok_or(Error::ComponentNotFound(component.to_string()))?;

        Ok(module.get(component, object)?)
    }

    pub fn set(&self, component: &str, object: &str, payload: &Payload) -> Result<()> {
        log::debug!("ModuleManager::set({}, {}, {})", component, object, payload);

        let module = self
            .modules
            .values()
            .find(|module| module.meta.components.contains(&component.to_string()))
            .ok_or(Error::ComponentNotFound(component.to_string()))?;

        module.set(component, object, payload)
    }
}

impl<T: Adapter> Module<T> {
    fn init<P: AsRef<Path>>(path: P) -> Result<Self> {
        let client = T::load(&path)?;
        let meta = client.meta()?;

        Ok(Self {
            path: path.as_ref().to_path_buf(),
            client: if meta.lifetime == Lifetime::Short {
                None
            } else {
                Some(client)
            },
            meta,
        })
    }

    /// Gets a value from the module.
    ///
    /// This function loads a new client session if the current one is `None` (lifetime is [`Lifetime::Short`]).
    /// If a new client is loaded (lifetime is [`Lifetime::Short`]), it will be closed when it is dropped.
    fn get(&self, component: &str, object: &str) -> Result<Payload> {
        log::debug!("Module::get({}, {})", component, object);

        if let Some(client) = &self.client {
            client.get(component, object)
        } else {
            let client = T::load(&self.path)?;
            client.get(component, object)
        }
    }

    /// Sets a value in the module.
    ///
    /// This function loads a new client session if the current one is `None` (lifetime is [`Lifetime::Short`]).
    /// If a new client is loaded (lifetime is [`Lifetime::Short`]), it will be closed when it is dropped.
    fn set(&self, component: &str, object: &str, payload: &Payload) -> Result<()> {
        log::debug!("Module::set({}, {}, {})", component, object, payload);

        if let Some(client) = &self.client {
            client.set(component, object, payload)
        } else {
            let client = T::load(&self.path)?;
            client.set(component, object, payload)
        }
    }
}

#[cfg(test)]
mod tests {
    use meta::Lifetime;

    use super::*;

    #[derive(Clone)]
    struct TestAdapter;

    impl Adapter for TestAdapter {
        fn load<P: AsRef<Path>>(_path: P) -> Result<Self> {
            Ok(Self)
        }

        fn meta(&self) -> Result<Metadata> {
            Ok(Metadata {
                // name: "test".to_string(),
                // description: None,
                // manufacturer: None,
                version: "1.2.3.4".into(),
                components: vec!["test".to_string()],
                lifetime: Lifetime::Short,
                // user_account: UserAccount::Root,
            })
        }

        fn get(&self, _component: &str, _object: &str) -> Result<Payload> {
            Ok(Payload::Null)
        }

        fn set(&self, _component: &str, _object: &str, _payload: &Payload) -> Result<()> {
            Ok(())
        }
    }

    #[test]
    fn test_module() {
        let module = Module::<TestAdapter>::init("/path/to/module.so").unwrap();

        // assert_eq!(module.meta.name, "test");
        assert_eq!(module.meta.version, "1.2.3.4".into());
        assert_eq!(module.meta.lifetime, Lifetime::Short);
        // assert_eq!(module.meta.user_account, UserAccount::Root);

        assert!(module.get("test", "test").is_ok());
        assert!(module.set("test", "test", &Payload::Null).is_ok());
    }
}
