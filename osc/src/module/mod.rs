use serde::{Deserialize, Serialize};

pub mod bindings;

// TODO: ModuleError type

// TODO: ModuleResult type

#[derive(Debug, Serialize, Deserialize)]
pub struct ModuleInfo {
    pub name: String,
    pub description: String,
    pub manufacturer: String,
    // pub version: Version,

    // TODO: change this to a "schema" type for the whole module (basically a MIM)
    pub components: Vec<String>,

    // TODO:
    // - version info
    // - lifetime
    // - license URI
    // - project URI
    // - user account
}

// REVIEW: instead of a Version struct use a custom wrapper for a string with special to/from conversions
#[derive(Debug, Serialize, Deserialize)]
pub struct Version {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
    pub tweak: Option<u32>,
}