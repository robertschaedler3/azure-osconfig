// These are required by the code generated via the `osc_codegen` macros.
pub use {serde, serde_json};

// Depend on osc_codegen and re-export everything in it.
// This allows users to just depend on osc and automatically get the derive functionality.
pub use osc_codegen::osc_component;

pub mod client;
pub mod config;
pub mod error;
pub mod log;
pub mod mim;
pub mod module;
