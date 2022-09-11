mod common;
mod osc_component;
// mod osc_module;
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

#[proc_macro_error]
#[proc_macro_attribute]
pub fn reported(_args: TokenStream, input: TokenStream) -> proc_macro::TokenStream {
    // NOTE: eventually this will need to wrap the object call for serialization, complex return types (Result, Option), async
    input
}

#[proc_macro_error]
#[proc_macro_attribute]
pub fn desired(_args: TokenStream, input: TokenStream) -> proc_macro::TokenStream {
    // NOTE: eventually this will need to wrap the object call for serialization, complex return types (Result, Option), async
    input
}

// TODO: there needs to be a #[osc_object] attribute for complex struct/enum types (for reported/desired properties),
// this should be possible to do automatically with in the component macro, but it may be better to have a separate attribute
