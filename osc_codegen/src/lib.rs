mod common;
mod osc_component;
mod osc_object;

use proc_macro::TokenStream;
use proc_macro_error::{proc_macro_error, ResultExt as _};

// #[proc_macro_derive(Module)]
// pub fn derive_osc_module(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
//     osc_module::derive::expand(input.into()).unwrap_or_abort().into()
// }

#[proc_macro_error]
#[proc_macro_attribute]
pub fn osc_component(args: TokenStream, input: TokenStream) -> proc_macro::TokenStream {
    osc_component::attr::expand(args.into(), input.into())
        .unwrap_or_abort()
        .into()
}
