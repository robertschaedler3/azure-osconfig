use std::fs;

use anyhow::{anyhow, Result};

use crate::{
    module::{Library, Session},
    Value,
};

const BIN: &str = "/usr/lib/osconfig";
const CLIENT: &str = "osc"; // TODO: use "official" client name

// TODO: move this into module.rs and refactor Library
#[derive(Clone)]
pub struct Module {
    // name: String,
    components: Vec<String>,
    library: Library,
    session: Option<Session>,
}

#[derive(Clone)]
pub struct Platform {
    modules: Vec<Module>,
}

impl Platform {
    pub fn load() -> Result<Self> {
        let mut modules = Vec::new();

        for entry in fs::read_dir(BIN)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().unwrap() == "so" {
                let library = Library::load(path)?;
                let info = library.info(CLIENT)?;
                let module = Module {
                    library,
                    components: info.components,
                    session: None,
                };
                modules.push(module);
            }
        }

        for module in &mut modules {
            // TODO: better error handling here
            module.session = Some(module.library.open(CLIENT, 0)?);
        }

        Ok(Self { modules })
    }

    pub fn get(&self, component: &str, object: &str) -> Result<Value> {
        let module = self
            .modules
            .iter()
            .find(|module| module.components.contains(&component.to_string()));

        if let Some(module) = module {
            if let Some(session) = module.session.clone() {
                let (status, payload) = module.library.get(&session, component, object)?;
                if status == 0 {
                    Ok(serde_json::from_str(&payload)?)
                } else {
                    Err(anyhow!("get({}.{}) failed: {}", component, object, status))
                }
            } else {
                Err(anyhow!("module session not open"))
            }
        } else {
            Err(anyhow!("Module not found"))
        }
    }

    pub fn set(&self, component: &str, object: &str, value: &Value) -> Result<()> {
        let module = self
            .modules
            .iter()
            .find(|module| module.components.contains(&component.to_string()));

        if let Some(module) = module {
            if let Some(session) = module.session.clone() {
                let payload = serde_json::to_string(value)?;
                let size = payload.len();
                let status = module
                    .library
                    .set(&session, component, object, &payload, size)?;
                if status == 0 {
                    Ok(())
                } else {
                    Err(anyhow!(
                        "set({} {} {:?}) failed: {}",
                        component,
                        object,
                        value,
                        status
                    ))
                }
            } else {
                Err(anyhow!("Module not open"))
            }
        } else {
            Err(anyhow!("Module not found"))
        }
    }
}

impl Drop for Platform {
    fn drop(&mut self) {
        for module in &mut self.modules {
            if let Some(session) = module.session.clone() {
                let _ = module.library.close(session);
            }
        }
    }
}
