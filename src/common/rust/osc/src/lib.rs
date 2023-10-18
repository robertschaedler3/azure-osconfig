// These are required by the code generated via the `osc_codegen` macros.
#[doc(hidden)]
pub use {serde, serde_json}; // REVIEW: this isnt working as expected

// Depend on osc_codegen and re-export everything in it.
// This allows users to just depend on osc and automatically get the macro functionality.
pub use osc_codegen::osc_module;

// TODO: make logging a cargo feature (also in codegen)
pub mod log;
pub mod module;
