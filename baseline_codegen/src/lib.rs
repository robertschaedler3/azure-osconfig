use proc_macro::TokenStream;
use proc_macro_type_name::ToTypeName;
use quote::quote;
use syn::{parse_macro_input, ItemFn};

// struct Params {
//     params: HashSet<Ident>,
// }

// impl

#[proc_macro_attribute]
pub fn check(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);

    let name = input.sig.ident;
    let params = input.sig.inputs;
    let body = input.block;

    let struct_name = (&name).to_type_ident(name.span());

    // Find all occurances of arguments in the body and replace them with the struct field
    // This cannot be done with a simple replace/tokens becasue other variables may have the same name

    // TODO: convert params into fields for struct

    // TODO: preserve visibility of function -> struct
    // TODO: handle lifetimes for args/struct
    let output = quote! {
        #[derive(Debug, serde::Deserialize)]
        pub struct #struct_name {
            #params
        }

        impl Check for #struct_name {
            fn check(&self) -> Result<bool> {
                log::debug!("Check: {:?}", self);
                #body
            }
        }
    };

    output.into()
}
