use std::fs;

use anyhow::{anyhow, Result};
use osc::module::{Library, Session};

static BIN: &str = "/usr/lib/osconfig";
static CLIENT_NAME: &str = "osc"; // TODO: use "official" client name

#[derive(Clone)]
pub struct Platform {
    modules: Vec<Module>,
}

#[derive(Clone)]
pub struct Module {
    // name: String,
    components: Vec<String>,
    library: Library,
    session: Option<Session>,
}

impl Platform {
    pub fn load() -> Result<Self> {
        let mut modules = Vec::new();

        for entry in fs::read_dir(BIN)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().unwrap() == "so" {
                let library = Library::load(path)?;
                let info = library.info(CLIENT_NAME)?;
                let module = Module {
                    // name: info.name,
                    components: info.components,
                    library,
                    session: None,
                };
                modules.push(module);
            }
        }

        for module in &mut modules {
            module.session = Some(module.library.open(CLIENT_NAME, 0)?);
        }

        Ok(Self { modules })
    }

    // TODO: return module type instead of string
    pub fn get(&self, component: &str, object: &str) -> Result<String> {
        let module = self
            .modules
            .iter()
            .find(|module| module.components.contains(&component.to_string()));
        if let Some(module) = module {
            if let Some(session) = module.session.clone() {
                let (status, payload) = module.library.get(session, component, object)?;
                if status == 0 {
                    Ok(payload)
                } else {
                    Err(anyhow!(
                        "MmiGet({} {}) failed: {}",
                        component,
                        object,
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

    pub fn set(&self, component: &str, object: &str, value: &str) -> Result<()> {
        let module = self
            .modules
            .iter()
            .find(|module| module.components.contains(&component.to_string()));
        if let Some(module) = module {
            if let Some(session) = module.session.clone() {
                let status =
                    module
                        .library
                        .set(session, component, object, value, value.len() as i32)?;
                if status == 0 {
                    Ok(())
                } else {
                    Err(anyhow!(
                        "MmiSet({} {} {}) failed: {}",
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
