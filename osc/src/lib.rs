// These are required by the code generated via the `osc_codegen` macros.
pub use {libc, serde, serde_json};

// Depend on osc_codegen and re-export everything in it.
// This allows users to just depend on osc and automatically get derive/macro functionality.
pub use osc_codegen::osc_module;

mod log;

pub mod config;
pub mod error;
pub mod mim;
pub mod module;

pub use crate::log::init_logger;