pub(crate) mod attr;
// pub(crate) mod derive;

use proc_macro2::Span;
use quote::{quote, ToTokens};
use syn::{
    ext::IdentExt as _,
    parse::{Parse, ParseStream},
    token,
};

use crate::common::filter_attrs;
use crate::osc_object;

/// [`diagnostic::Scope`] of errors for `#[osc_component]` macro.
// const ERR: diagnostic::Scope = diagnostic::Scope::ComponentAttr;

/// Available arguments behind `#[osc_component]` attribute.
#[derive(Debug, Default)]
pub(crate) struct Attr {
    /// Explictly specified name of this component.
    ///
    /// If [`None`] then the name will be derived from the type name.
    pub(crate) name: Option<String>,
    // TODO:
    // rename_fields: Option<renam::Policy>,
}

impl Parse for Attr {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let mut out = Self::default();
        while !input.is_empty() {
            let ident = input.call(syn::Ident::parse_any)?;
            match ident.to_string().as_str() {
                "name" => {
                    input.parse::<token::Eq>()?;
                    let name = input.parse::<syn::LitStr>()?;
                    out.name = Some(name.value());
                }
                _ => return Err(syn::Error::new(ident.span(), "unknown argument")),
            }
        }
        Ok(out)
    }
}

impl Attr {
    /// Tries to merge two [`Attr`]s into a single one, reporting about
    /// duplicates, if any.
    fn try_merge(self, mut another: Self) -> syn::Result<Self> {
        if self.name.is_some() && another.name.is_some() {
            return Err(syn::Error::new(
                Span::call_site(),
                "duplicate `name` attribute",
            ));
        }
        if self.name.is_some() {
            another.name = self.name;
        }
        Ok(another)
    }

    /// Parses [`Attr`] from the given multiple `name`d [`syn::Attribute`]s
    /// placed on a struct or impl block definition.
    pub(crate) fn from_attrs(name: &str, attrs: &[syn::Attribute]) -> syn::Result<Self> {
        let attr = filter_attrs(name, attrs)
            .map(|attr| attr.parse_args())
            .try_fold(Self::default(), |prev, curr| prev.try_merge(curr?))?;

        Ok(attr)
    }
}

struct Definition {
    pub(crate) name: String,
    // pub(crate) ty: syn::Type,
    pub(crate) ident: syn::Ident,
    pub(crate) reported_objects: Vec<osc_object::Definition>,
    pub(crate) desired_objects: Vec<osc_object::Definition>,
}

impl ToTokens for Definition {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let name = &self.name;

        let ident = &self.ident;
        let reported_objects = &self.reported_objects;
        let desired_objects = &self.desired_objects;

        let reported_objects = reported_objects.iter().map(|o| {
            let name = &o.name;
            let ident = &o.ident;
            let _ty = &o.ty;
            quote! {
                #name => Ok(serde_json::to_value(&self.#ident()).unwrap()),
            }
        });

        // TODO: desired objects should have a return code (not just an int, maybe Result<_, Error>)
        // how to interpret arbitrary return types as success/failure
        let desired_objects = desired_objects.iter().map(|o| {
            let name = &o.name;
            let ident = &o.ident;
            quote! {
                #name => self.#ident(value),
            }
        });

        tokens.extend(quote! {
            impl #ident {
            // REVIEW: why doesn't this work
            // impl ::osc::module::Component for #ident {
                fn reported(&self, object: &str) -> Result<osc::module::Object, osc::error::Error> {
                    match object {
                        #(#reported_objects)*
                        _ => Err(osc::error::Error::from(format!("unknown object: {}", object))),
                    }
                }

                fn desired(&mut self, object: &str, value: &str) {
                    match object {
                        #(#desired_objects)*
                        _ => {}
                    }
                }
            }
        });

        // TODO: move the module generation (and eventually MMI generation) somewhere else later
        // TODO: get the module name from #[osc_component(module = "name")] for generating multi-component modules
        tokens.extend(quote! {
            struct MyModule {
                component: #ident,
            }

            impl ::osc::module::Module for MyModule {
                fn new(_: &str, _: u32) -> Self {
                    Self {
                        component: #ident::default(),
                    }
                }

                fn get(&self, component: &str, object: &str) -> Result<osc::module::Object, osc::error::Error> {
                    match component {
                        #name => self.component.reported(object),
                        _ => Err(osc::error::Error::from(format!("unknown component: {}", component))),
                    }
                }

                fn set(&mut self, component: &str, object: &str, value: &str) {
                    match component {
                        #name => self.component.desired(object, value),
                        _ => println!("unknown component: {}", component),
                    }
                }
            }
        });
    }
}
