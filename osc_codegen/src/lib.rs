mod common;
mod osc_module;
mod osc_object;

use proc_macro::TokenStream;
use proc_macro_error::{proc_macro_error, ResultExt as _};

/// `#[osc_module]` macro for generating an [OSConfig module][1] implementation
/// for structs with compatible `#[reported]` and `#[desired]` property resolvers.
///
/// It enables you to write property resolvers for a single-component module by
/// declaring a regular Rust `impl` block for a struct. Under the hood, the macro
/// implements [`OscType`]/[`OscValue`] traits.
///
/// ```rust
/// use osc::osc_module;
///
/// // TODO:
/// ```
///
/// [1]: https://github.com/Azure/azure-osconfig/blob/main/docs/modules.md
#[proc_macro_error]
#[proc_macro_attribute]
pub fn osc_module(args: TokenStream, input: TokenStream) -> proc_macro::TokenStream {
    osc_module::attr::expand(args.into(), input.into())
        .unwrap_or_abort()
        .into()
}
