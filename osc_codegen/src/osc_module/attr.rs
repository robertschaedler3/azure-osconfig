use std::mem;

use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{ext::IdentExt as _, parse_quote, spanned::Spanned};

use crate::common::{
    diagnostic,
    parse::{self, TypeExt as _},
};
use crate::osc_object;

use super::{Attr, Definition};

/// [`diagnostic::Scope`] of errors for `#[osc_module]` macro.
const ERR: diagnostic::Scope = diagnostic::Scope::ComponentAttr;

/// Expands `#[osc_module]` macro into generated code.
pub fn expand(attr_args: TokenStream, body: TokenStream) -> syn::Result<TokenStream> {
    if let Ok(mut ast) = syn::parse2::<syn::ItemImpl>(body) {
        if ast.trait_.is_none() {
            let impl_attrs = parse::attr::unite(("osc_module", &attr_args), &ast.attrs);
            ast.attrs = parse::attr::strip("osc_module", ast.attrs);
            return expand_on_impl(Attr::from_attrs("osc_module", &impl_attrs)?, ast);
        }
    }

    Err(syn::Error::new(
        Span::call_site(),
        "#[osc_module] attribute is applicable to non-trait `impl` blocks only",
    ))
}

/// Expands `#[osc_module]` macro placed on an implmentation block.
pub(crate) fn expand_on_impl(attr: Attr, mut ast: syn::ItemImpl) -> syn::Result<TokenStream> {
    let type_span = ast.self_ty.span();
    let type_ident = ast.self_ty.topmost_ident().ok_or_else(|| {
        ERR.custom_error(type_span, "could not determine ident for the `impl` type")
    })?;

    // TODO: if using type_ident for name, it should be checked and fixed according to a naming policy
    let name = attr
        .name
        .clone()
        .unwrap_or_else(|| type_ident.unraw().to_string());

        let reported_objects: Vec<_> = ast
        .items
        .iter_mut()
        .filter_map(|item| {
            if let syn::ImplItem::Method(method) = item {
                if method.attrs.iter().any(|a| a.path.is_ident("reported")) {
                    parse_reported(method)
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect();

        let desired_objects: Vec<_> = ast
            .items
            .iter_mut()
            .filter_map(|item| {
                if let syn::ImplItem::Method(method) = item {
                    if method.attrs.iter().any(|a| a.path.is_ident("desired")) {
                        parse_desired(method)
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect();

    let generated_code = Definition {
        name,
        ident: type_ident.unraw(),
        // ty: ast.self_ty.unparenthesized().clone(),
        reported_objects,
        desired_objects,
    };

    Ok(quote! {
        #ast
        #generated_code
    })
}

/// Parses an object from an item in an impl block.
///
/// Returns `None` if the item is not an object.
fn parse_reported(method: &mut syn::ImplItemMethod) -> Option<osc_object::Definition> {
    let method_attrs = method.attrs.clone();

    method.attrs = mem::take(&mut method.attrs)
        .into_iter()
        .filter(|attr| !attr.path.is_ident("reported"))
        .collect();

    // TODO: make named attributes optional
    let attr = osc_object::Attr::from_attrs("reported", &method_attrs)
    .map_err(|e| proc_macro_error::emit_error!(e))
        .ok()?;

    let method_ident = &method.sig.ident;

    // TODO: if using method_ident for name, it should be checked and fixed according to a naming policy
    let name = attr
    .name
    .clone()
    .unwrap_or_else(|| method_ident.unraw().to_string());

    let ty = match &method.sig.output {
        syn::ReturnType::Default => {
            // TODO: error
            parse_quote! { () }
        },
        syn::ReturnType::Type(_, ty) => ty.unparenthesized().clone(),
    };

    Some(osc_object::Definition {
        name,
        ty,
        ident: method_ident.clone(),
    })
}

fn parse_desired(method: &mut syn::ImplItemMethod) -> Option<osc_object::Definition> {
    let method_attrs = method.attrs.clone();

    method.attrs = mem::take(&mut method.attrs)
        .into_iter()
        .filter(|attr| !attr.path.is_ident("desired"))
        .collect();

    // TODO: make named attributes optional
    let attr = osc_object::Attr::from_attrs("desired", &method_attrs)
        .map_err(|e| proc_macro_error::emit_error!(e))
        .ok()?;

    let method_ident = &method.sig.ident;

    // list arguements of the method
    let mut args = Vec::new();
    for arg in &method.sig.inputs {
        if let syn::FnArg::Typed(arg) = arg {
            args.push(arg.ty.clone());
        }
    }

    // the only args should be &mut self and one other arg
    if args.len() != 1 {
        // TODO: error
        return None;
    }

    // TODO: if using method_ident for name, it should be checked and fixed according to a naming policy
    let name = attr
        .name
        .clone()
        .unwrap_or_else(|| method_ident.unraw().to_string());

    Some(osc_object::Definition {
        name,
        ty: *args[0].clone(),
        ident: method_ident.clone(),
    })
}