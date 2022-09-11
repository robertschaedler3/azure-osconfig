use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{ext::IdentExt as _, spanned::Spanned};

use crate::common::{parse::{self, TypeExt as _}, diagnostic};
use crate::osc_object;

use super::{Attr, Definition};

/// [`diagnostic::Scope`] of errors for `#[osc_component]` macro.
const ERR: diagnostic::Scope = diagnostic::Scope::ComponentAttr;

/// Expands `#[osc_component]` macro into generated code.
pub fn expand(attr_args: TokenStream, body: TokenStream) -> syn::Result<TokenStream> {
    if let Ok(mut ast) = syn::parse2::<syn::ItemImpl>(body) {
        if ast.trait_.is_none() {
            let impl_attrs = parse::attr::unite(("osc_component", &attr_args), &ast.attrs);
            ast.attrs = parse::attr::strip("osc_component", ast.attrs);
            return expand_on_impl(Attr::from_attrs("osc_component", &impl_attrs)?, ast);
        }
    }

    Err(syn::Error::new(
        Span::call_site(),
        "#[osc_component] attribute is applicable to non-trait `impl` blocks only",
    ))
}

/// Expands `#[osc_component]` macro placed on an implmentation block.
pub(crate) fn expand_on_impl(attr: Attr, ast: syn::ItemImpl) -> syn::Result<TokenStream> {
    let type_span = ast.self_ty.span();
    let type_ident = ast.self_ty.topmost_ident().ok_or_else(|| {
        ERR.custom_error(type_span, "could not determine ident for the `impl` type")
    })?;

    let name = attr
        .name
        .clone()
        .unwrap_or_else(|| type_ident.unraw().to_string());

    let objects: Vec<_> = ast
        .items
        .iter()
        .filter_map(|item| parse_object(item))
        .collect();

    let reported_objects = objects.clone().into_iter().filter(|o| o.kind == osc_object::Kind::Reported).collect();
    let desired_objects = objects.clone().into_iter().filter(|o| o.kind == osc_object::Kind::Desired).collect();

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
fn parse_object(item: &syn::ImplItem) -> Option<osc_object::Definition> {
    match item {
        syn::ImplItem::Method(method) => {
            let name = method.sig.ident.to_string();
            let ident = method.sig.ident.clone();

            let reported = method.attrs.iter().any(|attr| attr.path.is_ident("reported"));
            let desired = method.attrs.iter().any(|attr| attr.path.is_ident("desired"));

            if reported && desired {
                ERR.emit_custom(
                    method.span(),
                    "method cannot be both reported and desired",
                );
            }

            let kind = if reported {
                osc_object::Kind::Reported
            } else {
                osc_object::Kind::Desired
            };

            Some(osc_object::Definition { name, ident, kind })
        }
        _ => None,
    }
}
