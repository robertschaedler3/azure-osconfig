use serde::{Serialize, Deserialize};

const CONFIG_NAME: &str = "osc.conf";

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct Config {
    pub session: String,
}

impl Config {
    pub fn load() -> Self {
        confy::load::<Self>(CONFIG_NAME).unwrap_or_default()
    }

    // pub fn store(&self) {

    // }
}