//! Code generation for [OSConfig modules][1].
//!
//! [1]: TODO:

use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{
    parse::{Parse, ParseStream},
    token,
};

use crate::common::{
    filter_attrs,
    parse::{
        attr::{err, OptionExt as _},
        ParseBufferExt as _,
    },
    SpanContainer,
};

pub mod attr;
mod property;

// TODO: use the Manufacturer and UserAccount attr too
use self::attr::{Lifetime, Version};

/// Available arguments behind `#[osc]` (or `#[osc_module]`) attribute
/// when generating code for an [OSConfig module][1].
///
/// [1]: TODO:
#[derive(Debug, Default)]
pub(crate) struct Attr {
    // TODO: documentation for each property
    pub(crate) name: Option<SpanContainer<String>>,
    pub(crate) description: Option<SpanContainer<String>>,
    pub(crate) manufacturer: Option<SpanContainer<String>>,
    pub(crate) version: Option<SpanContainer<Version>>,
    pub(crate) lifetime: Option<SpanContainer<Lifetime>>,
    // TODO:
    // pub(crate) user_account: Option<SpanContainer<u32>>,
}

impl Parse for Attr {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let mut out = Self::default();
        while !input.is_empty() {
            let ident = input.parse_any_ident()?;
            match ident.to_string().as_str() {
                "name" => {
                    input.parse::<token::Eq>()?;
                    let name = input.parse::<syn::LitStr>()?;
                    out.name
                        .replace(SpanContainer::new(
                            ident.span(),
                            Some(name.span()),
                            name.value(),
                        ))
                        .none_or_else(|_| err::dup_arg(&ident))?
                }
                "desc" | "description" => {
                    input.parse::<token::Eq>()?;
                    let desc = input.parse::<syn::LitStr>()?;
                    out.description
                        .replace(SpanContainer::new(
                            ident.span(),
                            Some(desc.span()),
                            desc.value(),
                        ))
                        .none_or_else(|_| err::dup_arg(&ident))?
                }
                "manufacturer" => {
                    // TODO: use attr definition
                    input.parse::<token::Eq>()?;
                    let manufacturer = input.parse::<syn::LitStr>()?;
                    out.manufacturer
                        .replace(SpanContainer::new(
                            ident.span(),
                            Some(manufacturer.span()),
                            manufacturer.value(),
                        ))
                        .none_or_else(|_| err::dup_arg(&ident))?
                }
                "version" => {
                    input.parse::<token::Eq>()?;
                    let version = input.parse::<Version>()?;
                    out.version
                        .replace(SpanContainer::new(ident.span(), None, version))
                        .none_or_else(|_| err::dup_arg(&ident))?
                }
                "lifetime" => {
                    input.parse::<token::Eq>()?;
                    let lifetime = input.parse::<Lifetime>()?;
                    out.lifetime
                        .replace(SpanContainer::new(ident.span(), None, lifetime))
                        .none_or_else(|_| err::dup_arg(&ident))?
                }
                name => {
                    return Err(err::unknown_arg(&ident, name));
                }
            }
            input.try_parse::<token::Comma>()?;
        }
        Ok(out)
    }
}

impl Attr {
    /// Tries to merge two [`Attr`]s into a single one, reporting about
    /// duplicates, if any.
    fn try_merge(self, mut another: Self) -> syn::Result<Self> {
        Ok(Self {
            name: try_merge_opt!(name: self, another),
            description: try_merge_opt!(description: self, another),
            manufacturer: try_merge_opt!(manufacturer: self, another),
            version: try_merge_opt!(version: self, another),
            lifetime: try_merge_opt!(lifetime: self, another),
        })
    }

    /// Parses [`Attr`] from the given multiple `name`d [`syn::Attribute`]s
    /// placed on a struct or impl block definition.
    pub(crate) fn from_attrs(name: &str, attrs: &[syn::Attribute]) -> syn::Result<Self> {
        filter_attrs(name, attrs)
            .map(|attr| attr.parse_args())
            .try_fold(Self::default(), |prev, curr| prev.try_merge(curr?))
    }
}

/// Definition of a module for code generation.
// #[derive(Debug)]
pub(crate) struct Definition {
    /// Name of this [module][1].
    ///
    /// [1]: TODO:
    pub(crate) name: String,

    /// Rust type that this [module][1] is represented with.
    ///
    /// It should contain all its generics, if any.
    ///
    /// [1]: TODO:
    pub(crate) ty: syn::Type,

    // Generics of the Rust type that this [module][1] is implemented
    // for.
    //
    // [1]: TODO:
    // pub(crate) generics: syn::Generics,

    // pub(crate) description: Option<Description>,
    /// Description of this module to put into the module metadata.
    pub(crate) description: Option<String>,

    /// Defined [properties][2] of this [OSConfig module][1].
    ///
    /// [1]: TODO:
    /// [2]: TODO:
    pub(crate) props: Vec<property::Definition>,

    /// TODO:
    pub(crate) manufacturer: Option<String>,

    /// TODO:
    pub(crate) version: Version,

    /// TODO:
    pub(crate) lifetime: Lifetime,
}

impl Definition {
    fn impl_osc_module_init_tokens(&self) -> TokenStream {
        let name = &self.name;

        quote! {
            #[::ctor::ctor]
            fn __ctor__() {
                ::osc::log::init();
                ::log::info!("loaded: {}", #name);
            }

            #[::ctor::dtor]
            fn __dtor__() {
                ::log::info!("unloaded: {}", #name);
            }
        }
    }

    fn impl_meta_tokens(&self) -> TokenStream {
        let name = &self.name;

        let description = self
            .description
            .as_ref()
            .map(|s| quote! { Some(#s.to_string()) })
            .unwrap_or_else(|| quote! { None });

        // TODO: custom to TokenStream impl
        let manufacturer = self
            .manufacturer
            .as_ref()
            .map(|s| quote! { Some(#s.to_string()) })
            .unwrap_or_else(|| quote! { None });

        let version = &self.version;
        let lifetime = &self.lifetime;

        quote! {
            ::osc::module::Meta {
                name: #name.to_string(),
                description: #description,
                manufacturer: #manufacturer,
                version: #version,
                components: vec![#name.to_string()],
                lifetime: #lifetime,
                user_account: ::osc::module::UserAccount::Root,
            }
        }
    }

    fn impl_osc_module_meta_tokens(&self) -> TokenStream {
        let meta = self.impl_meta_tokens();

        quote! {
            #[no_mangle]
            pub extern "C" fn MmiGetInfo(
                client: *const ::std::ffi::c_char,
                payload: *mut *mut ::std::ffi::c_char,
                payload_size: *mut ::std::ffi::c_int,
            ) -> ::std::ffi::c_int {
                if client.is_null() {
                    ::log::error!("null client name");
                    return ::libc::EINVAL;
                }

                if payload.is_null() {
                    ::log::error!("null payload");
                    return ::libc::EINVAL;
                }

                if payload_size.is_null() {
                    ::log::error!("null payload size");
                    return ::libc::EINVAL;
                }

                unsafe {
                    *payload = ::std::ptr::null_mut();
                    *payload_size = 0;
                }

                let meta = #meta;

                let json = ::serde_json::to_string(&meta).unwrap();
                let json = ::std::ffi::CString::new(json.as_str()).unwrap();
                let size = json.as_bytes().len() as ::std::ffi::c_int;

                unsafe {
                    *payload = json.into_raw();
                    *payload_size = size as ::std::ffi::c_int;
                };

                0
            }
        }
    }

    fn impl_osc_module_open_tokens(&self) -> TokenStream {
        let ty = &self.ty;

        quote! {
            #[no_mangle]
            pub extern "C" fn MmiOpen(
                client: *const ::std::ffi::c_char,
                max_size: ::std::ffi::c_uint
            ) -> *mut ::std::ffi::c_void {
                if client.is_null() {
                    ::log::error!("null client name");
                    return std::ptr::null_mut();
                }

                let module = Box::new(#ty::default()); // TODO: do this without requiring default
                Box::into_raw(module) as *mut ::std::ffi::c_void
            }
        }
    }

    fn impl_osc_module_close_tokens(&self) -> TokenStream {
        let ty = &self.ty;
        quote! {
            #[no_mangle]
            pub extern "C" fn MmiClose(handle: *mut ::std::ffi::c_void) {
                if !handle.is_null() {
                    let _ = unsafe { Box::from_raw(handle as *mut #ty) };
                }
            }
        }
    }

    fn impl_osc_module_set_tokens(&self) -> TokenStream {
        let (name, ty) = (&self.name, &self.ty);

        let desired = self
            .props
            .iter()
            .filter(|prop| prop.is_desired())
            .map(|prop| prop.method_resolve_desired_prop_tokens(ty));

        quote! {
            #[no_mangle]
            pub extern "C" fn MmiSet(
                handle: *mut ::std::ffi::c_void,
                component: *const ::std::ffi::c_char,
                property: *const ::std::ffi::c_char,
                payload: *mut ::std::ffi::c_char,
                payload_size: ::std::ffi::c_int,
            ) -> ::std::ffi::c_int {
                if handle.is_null() {
                    ::log::error!("null handle");
                    return ::libc::EINVAL;
                }

                if component.is_null() {
                    ::log::error!("null component");
                    return ::libc::EINVAL;
                }

                if property.is_null() {
                    ::log::error!("null property");
                    return ::libc::EINVAL;
                }

                let component = unsafe { std::ffi::CStr::from_ptr(component) };
                let component = component.to_str().unwrap(); // TODO: use `?` instead

                if #name != component {
                    ::log::error!("invalid component: {}", component);
                    return ::libc::EINVAL;
                }

                let property = unsafe { std::ffi::CStr::from_ptr(property) };
                let property = property.to_str().unwrap(); // TODO: use `?` instead

                let payload = unsafe { std::slice::from_raw_parts(payload as *const u8, payload_size as usize) };
                let payload = String::from_utf8_lossy(payload).to_string();

                let module: &mut #ty = unsafe { &mut *(handle as *mut #ty) };

                match property {
                    #(#desired)*
                    _ => {
                        ::log::error!("invalid property: {}.{}", component, property);
                    }
                }

                ::libc::EXIT_SUCCESS
            }
        }
    }

    fn impl_osc_module_get_tokens(&self) -> TokenStream {
        let (name, ty) = (&self.name, &self.ty);

        let reported = self
            .props
            .iter()
            .filter(|prop| prop.is_reported())
            .map(|prop| prop.method_resolve_reported_prop_tokens(ty));

        quote! {
            #[no_mangle]
            pub extern "C" fn MmiGet(
                handle: *mut ::std::ffi::c_void,
                component: *const ::std::ffi::c_char,
                property: *const ::std::ffi::c_char,
                payload: *mut *mut ::std::ffi::c_char,
                payload_size: *mut ::std::ffi::c_int,
            ) -> ::std::ffi::c_int {
                if handle.is_null() {
                    ::log::error!("null handle");
                    return ::libc::EINVAL;
                }

                if component.is_null() {
                    ::log::error!("null component");
                    return ::libc::EINVAL;
                }

                if property.is_null() {
                    ::log::error!("null property");
                    return ::libc::EINVAL;
                }

                if payload.is_null() {
                    ::log::error!("null payload");
                    return ::libc::EINVAL;
                }

                if payload_size.is_null() {
                    ::log::error!("null payload size");
                    return ::libc::EINVAL;
                }

                unsafe {
                    *payload = std::ptr::null_mut();
                    *payload_size = 0;
                }

                let component = unsafe { ::std::ffi::CStr::from_ptr(component) };
                let component = component.to_str().unwrap(); // TODO: use `?` instead

                if #name != component {
                    ::log::error!("invalid component: {}", component);
                    return ::libc::EINVAL;
                }

                let property = unsafe { ::std::ffi::CStr::from_ptr(property) };
                let property = property.to_str().unwrap(); // TODO: use `?` instead

                // REVIEW: it is unlear in the resolver what the name of this variable is...
                let module: &#ty = unsafe { &*(handle as *const #ty) };

                let res = match property {
                    #(#reported)*
                    _ => {
                        // REVIEW: this is ugly and not very rust-y
                        ::osc::module::PropertyResult::Err(::osc::module::PropertyError::new(format!("invalid property: {}.{}", component, property)))
                    }
                };

                match res {
                    Ok(val) => {
                        // REVIEW: val.to_string() feels weird...
                        let json = ::std::ffi::CString::new(val.to_string().unwrap()).unwrap();
                        let size = json.as_bytes().len() as ::std::ffi::c_int;

                        unsafe {
                            *payload = json.into_raw();
                            *payload_size = size as ::std::ffi::c_int;
                        };

                        ::libc::EXIT_SUCCESS
                    }
                    Err(err) => {
                        ::log::error!("{}", err);
                        -1 // TODO: map the error to Errno
                    }
                }
            }
        }
    }

    fn impl_osc_module_free_tokens(&self) -> TokenStream {
        quote! {
            // TODO:
        }
    }
}

impl ToTokens for Definition {
    fn to_tokens(&self, into: &mut TokenStream) {
        self.impl_osc_module_init_tokens().to_tokens(into);
        self.impl_osc_module_meta_tokens().to_tokens(into);
        self.impl_osc_module_open_tokens().to_tokens(into);
        self.impl_osc_module_close_tokens().to_tokens(into);
        self.impl_osc_module_set_tokens().to_tokens(into);
        self.impl_osc_module_get_tokens().to_tokens(into);
        self.impl_osc_module_free_tokens().to_tokens(into);
    }
}
