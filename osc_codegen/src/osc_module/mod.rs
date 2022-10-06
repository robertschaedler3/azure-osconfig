pub(crate) mod attr;
// pub(crate) mod derive;

use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{
    ext::IdentExt as _,
    parse::{Parse, ParseStream},
    token,
};

use crate::common::filter_attrs;
use crate::osc_object;

/// [`diagnostic::Scope`] of errors for `#[osc_module]` macro.
// const ERR: diagnostic::Scope = diagnostic::Scope::ComponentAttr;

/// Available arguments behind `#[osc_module]` attribute.
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

impl Definition {
    pub(crate) fn impl_component_struct_tokens(&self) -> TokenStream {
        let name = &self.name;
        let ident = &self.ident;

        // TODO: try to consolidate this code
        let reported_objects = &self.reported_objects;
        let desired_objects = &self.desired_objects;

        let reported_objects = reported_objects.iter().map(|o| {
            let name = &o.name;
            let ident = &o.ident;
            let _ty = &o.ty;
            // TODO: properly convert from serde result to osc result (with ? operator)
            quote! {
                #name => Ok(serde_json::to_value(&self.#ident()).unwrap()),
            }
        });

        // TODO: check (and validate/restrict) for return value from `desired` methods

        let desired_objects = desired_objects.iter().map(|o| {
            let name = &o.name;
            let ident = &o.ident;
            let ty = &o.ty;
            quote! {
                #name => Ok(self.#ident(serde_json::from_value::<#ty>(value).unwrap())),
            }
        });

        // TODO: handle complex return codes from property resolvers (e.g. `Result<_, _>`)

        quote! {
            impl osc::module::Component for #ident {
                fn name(&self) -> &str {
                    #name
                }

                fn reported(&self, name: &str) -> Result<osc::module::Object, osc::error::Error> {
                    match name {
                        #(#reported_objects)*
                        _ => Err(osc::error::Error::from(format!("unknown object: {}", name))),
                    }
                }

                fn desired(&mut self, name: &str, value: serde_json::Value) -> Result<(), osc::error::Error> {
                    match name {
                        #(#desired_objects)*
                        _ => Err(osc::error::Error::from(format!("unknown object: {}", name))),
                    }
                }
            }
        }
    }

    pub(crate) fn impl_module_tokens(&self) -> TokenStream {
        let ident = &self.ident;
        let module_ident = syn::Ident::new(&format!("{}Module", ident), ident.span());

        quote! {
            type #module_ident = osc::module::Module<#ident>;
        }
    }

    pub(crate) fn impl_module_interface_tokens(&self) -> TokenStream {
        let ident = &self.ident;
        let module_ident = syn::Ident::new(&format!("{}Module", ident), ident.span());

        quote! {
            #[no_mangle]
            pub extern "C" fn MmiOpen(client_name: *const libc::c_char, max_payload_size: libc::c_uint) -> osc::module::interface::Handle {
                if let Ok(module) = osc::module::interface::open::<#module_ident>(client_name, max_payload_size) {
                    Box::into_raw(Box::new(module)) as osc::module::interface::Handle
                } else {
                    // TODO: log error
                    println!("MmiOpen failed");
                    ptr::null_mut()
                }
            }

            #[no_mangle]
            pub extern "C" fn MmiClose(client_session: osc::module::interface::Handle) {
                osc::module::interface::close::<#module_ident>(client_session);
            }

            #[no_mangle]
            pub extern "C" fn MmiSet(
                client_session: osc::module::interface::Handle,
                component_name: *const libc::c_char,
                object_name: *const libc::c_char,
                payload: osc::module::interface::JsonString,
                payload_size_bytes: libc::c_int,
            ) -> libc::c_int {
                if let Err(err) = osc::module::interface::set::<#module_ident>(
                    client_session,
                    component_name,
                    object_name,
                    payload,
                    payload_size_bytes,
                ) {
                    // TODO: log error
                    println!("error: {}", err);

                    // TODO: convert error to appropriate error code
                    libc::EINVAL
                } else {
                    0
                }
            }

            #[no_mangle]
            pub extern "C" fn MmiGet(
                client_session: osc::module::interface::Handle,
                component_name: *const libc::c_char,
                object_name: *const libc::c_char,
                payload: *mut osc::module::interface::JsonString,
                payload_size_bytes: *mut libc::c_int,
            ) -> libc::c_int {
                if let Err(err) = osc::module::interface::get::<#module_ident>(
                    client_session,
                    component_name,
                    object_name,
                    payload,
                    payload_size_bytes,
                ) {
                    // TODO: log error
                    println!("error: {}", err);

                    // TODO: convert error to appropriate error code
                    libc::EINVAL
                } else {
                    0
                }
            }
        }
    }
}

impl ToTokens for Definition {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.impl_component_struct_tokens().to_tokens(tokens);
        self.impl_module_tokens().to_tokens(tokens);
        self.impl_module_interface_tokens().to_tokens(tokens);
    }
}
