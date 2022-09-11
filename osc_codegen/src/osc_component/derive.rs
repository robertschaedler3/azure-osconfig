use proc_macro2::TokenStream;
use proc_macro_error::ResultExt;
use quote::{ToTokens};
// use syn::{ext::IdentExt as _, parse_quote, spanned::Spanned as _};

use super::{Definition};
use crate::osc_object::{self, Kind};

pub fn expand(input: TokenStream) -> syn::Result<TokenStream> {
    let ast = syn::parse2::<syn::DeriveInput>(input).unwrap_or_abort();

    match &ast.data {
        syn::Data::Struct(_) => expand_struct(ast),
        _ => Err(syn::Error::new_spanned(ast, "only structs are supported")),
    }
    .map(ToTokens::into_token_stream)
}

/// Expand a struct into a generated `#[derive(Component)]` implementation.
fn expand_struct(ast: syn::DeriveInput) -> syn::Result<Definition> {
    // TODO: also register the component impl with the for = <module> attribute

    let name = ast.ident;
    // ast.data.

    // parse the impl block for the struct
    let mut objects = Vec::new();


    // get a list of all the methods in the struct
    let methods = match &ast.data {
        syn::Data::Struct(syn::DataStruct { fields, .. }) => match fields {
            syn::Fields::Named(syn::FieldsNamed { named, .. }) => named
                .iter()
                .map(|f| f.ident.as_ref().unwrap())
                .collect::<Vec<_>>(),
            _ => unimplemented!(),
        },
        _ => unimplemented!(),
    };

    // for each method, create a new osc_object::Definition
    let objects = methods
        .iter()
        .map(|method| {
            let name = method.to_string();
            let ident = syn::Ident::new(&name, proc_macro2::Span::call_site());

            // // check the method for the #[reported] or #[desired] attribute
            // let kind = match ast.attrs.iter().find(|a| a.path.is_ident("reported")) {
            //     Some(_) => Kind::Reported,
            //     None => Kind::Desired,
            // };

            Ok(osc_object::Definition {
                name,
                ident,
                // ty: parse_quote!(String),
                kind: Kind::Reported,
            })
        })
        .collect::<syn::Result<Vec<_>>>()?;

    Ok(Definition {
        name: name.to_string(),
        // ty: parse_quote! { #name },
        objects,
    })
}

// fn parse_object(field: &syn::Field) -> Option<ObjectDefinition> {
//     // TODO: parse attributes (ignored, rename, etc.)

//     let name = field.ident.as_ref().unwrap().unraw().to_string();
//     let field_ident = field.ident.as_ref().unwrap();

//     // TODO: validate name

//     let ty = field.ty.clone(); // TODO: unparenthesize and lifetimes_anonymize ???

//     Some(ObjectDefinition {
//         name,
//         ty,
//     })
// }