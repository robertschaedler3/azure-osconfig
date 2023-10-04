use std::{collections::HashMap, sync::{Mutex, Arc}};

use anyhow::{Result, Context};
use serde_json::Value;

use crate::module::{Manifest, Module};

mod action;
mod module;

pub type Platform = Arc<Mutex<Registry>>;

pub fn load(path: &str) -> Result<Platform> {
    let mut registry = Registry::default();

    for entry in std::fs::read_dir(path)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            if let Err(err) = registry.load(path.to_str().context("Unable to convert path to string")?) {
                log::error!("Failed to load module: {}", err);
            }
        }
    }

    Ok(Arc::new(Mutex::new(registry)))
}

#[derive(Default)]
pub struct Registry {
    // Map of module names to module instances
    modules: HashMap<String, module::Module>
}

impl Registry {
    /// Loads a module given a path to a module manifest file
    pub fn load(&mut self, path: &str) -> Result<()> {
        let content = std::fs::read_to_string(path)?;
        let manifest = serde_yaml::from_str::<Manifest>(&content)?;
        let (name, module) = Module::new(manifest)?;

        self.modules.insert(name, module);

        Ok(())
    }

    /// Finds the module with the given name and resolves its value
    pub fn get(&self, module: &str, property: &str) -> Result<Value> {
        self.modules
            .get(module)
            .context("Failed to find module")?
            .resolve(property)
    }
}
