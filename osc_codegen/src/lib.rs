mod common;
mod osc_module;
mod osc_object;

use proc_macro::TokenStream;
use proc_macro_error::{proc_macro_error, ResultExt as _};

#[proc_macro_error]
#[proc_macro_attribute]
pub fn osc_module(args: TokenStream, input: TokenStream) -> proc_macro::TokenStream {
    osc_module::attr::expand(args.into(), input.into())
        .unwrap_or_abort()
        .into()
}
