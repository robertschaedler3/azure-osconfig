//! Common functions, definitions and extensions for parsing and code generation
//! of [OSConfig properties][1]
//!
//! [1]: TODO:
//!

use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    spanned::Spanned as _,
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

/// Available metadata (arguments) behind `#[osc]` attribute placed on a
/// [OSConfig property][1] definition.
///
/// [1]: TODO:
#[derive(Default)]
pub struct Attr {
    /// Explicitly specified name of this [OSConfig property][1].
    ///
    /// If [`None`], then `camelCased` Rust method name is used by default. // REVIEW: this is not true
    ///
    /// [1]: TODO:
    pub(crate) name: Option<SpanContainer<syn::LitStr>>,

    /// Explicitly specified marker indicating that this method (or struct
    /// field) should be omitted by code generation and not considered as the
    /// [OSConfig property][1] definition.
    ///
    /// [1]: TODO:
    // pub(crate) ignore: Option<SpanContainer<syn::Ident>>,

    /// TODO:
    pub(crate) access: Option<SpanContainer<AccessType>>,
}

#[derive(PartialEq)]
pub enum AccessType {
    Reported,
    Desired,
}

impl Parse for Attr {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let mut out = Self::default();
        while !input.is_empty() {
            let ident = input.parse::<syn::Ident>()?;
            match ident.to_string().as_str() {
                "name" => {
                    input.parse::<token::Eq>()?;
                    let name = input.parse::<syn::LitStr>()?;
                    out.name
                        .replace(SpanContainer::new(ident.span(), Some(name.span()), name))
                        .none_or_else(|_| err::dup_arg(&ident))?
                }
                "reported" | "read" => out
                    .access
                    .replace(SpanContainer::new(ident.span(), None, AccessType::Reported))
                    .none_or_else(|_| err::dup_arg(&ident))?,

                "desired" | "write" => out
                    .access
                    .replace(SpanContainer::new(ident.span(), None, AccessType::Desired))
                    .none_or_else(|_| err::dup_arg(&ident))?,
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
    /// Tries to merge two [`Attrs`]s into a single one, reporting about
    /// duplicates, if any.
    fn try_merge(self, mut another: Self) -> syn::Result<Self> {
        Ok(Self {
            name: try_merge_opt!(name: self, another),
            // description: try_merge_opt!(description: self, another),
            // deprecated: try_merge_opt!(deprecated: self, another),
            access: try_merge_opt!(access: self, another),
            // ignore: try_merge_opt!(ignore: self, another),
        })
    }

    /// Parses [`Attr`] from the given multiple `name`d [`syn::Attribute`]s
    /// placed on a [OSConfig property][1] definition.
    ///
    /// [1]: TODO:
    pub fn from_attrs(name: &str, attrs: &[syn::Attribute]) -> syn::Result<Self> {
        let mut attr = filter_attrs(name, attrs)
            .map(|attr| attr.parse_args())
            .try_fold(Self::default(), |prev, curr| prev.try_merge(curr?))?;

        // if let Some(ignore) = &attr.ignore {
        //     // if attr.name.is_some() || attr.description.is_some() || attr.deprecated.is_some() {
        //     if attr.name.is_some() {
        //         return Err(syn::Error::new(
        //             ignore.span(),
        //             "`ignore` attribute argument is not composable with any other arguments",
        //         ));
        //     }
        // }

        if let None = &attr.access {
            return Err(syn::Error::new(
                name.span(),
                "missing `reported` or `desired` attribute argument",
            ));
        }

        // if attr.description.is_none() {
        //     attr.description = Description::parse_from_doc_attrs(attrs)?;
        // }

        // if attr.deprecated.is_none() {
        //     attr.deprecated = deprecation::Directive::parse_from_deprecated_attr(attrs)?;
        // }

        Ok(attr)
    }
}

/// Representation of an [OSConfig property][1] for code generation.
///
/// [1]: TODO:
pub struct Definition {
    /// Rust type that this [OSConfig property][1] is represented by (method return
    /// type or struct field type).
    ///
    /// [1]: TODO:
    pub ty: syn::Type,

    /// Name of this [OSConfig property][1] in the schema.
    ///
    /// [1]: TODO:
    pub name: String,

    // pub(crate) deprecated: Option<deprecation::Directive>,
    /// Ident of the Rust method (or struct field) representing this
    /// [OSConfig property][1].
    ///
    /// [1]: TODO:
    pub ident: syn::Ident,

    /// Rust [`MethodArgument`]s required to call the method representing this
    /// [OSConfig property][1].
    ///
    /// If [`None`] then this [OSConfig property][1] is represented by a struct // REVIEW:
    /// field.
    ///
    /// [1]: TODO:
    pub arguments: Option<Vec<syn::Type>>,

    /// Indicator whether the Rust method representing this [OSConfig property][1]
    /// has a [`syn::Receiver`].
    ///
    /// [1]: TODO:
    pub has_receiver: bool,

    /// TODO:
    pub access: AccessType,
}

impl Definition {
    // REVIEW: need a better arg name than ty_name
    #[must_use]
    pub fn method_resolve_reported_prop_tokens(&self, ty_name: &syn::Type) -> TokenStream {
        let (name, mut ty, ident) = (&self.name, self.ty.clone(), &self.ident);

        // REVIEW: it is super unlear in that `module` is the variable name used in the caller's generated code
        let rcv = self.has_receiver.then(|| {
            quote! { &module }
        });

        quote! {
             #name => {
                let res = ::osc::module::IntoPropertyResult::into_result(
                    #ty_name::#ident(#rcv)
                );
                ::osc::module::IntoResolvable::into_resolvable(res)
            }
        }
    }

    #[must_use]
    pub fn method_resolve_desired_prop_tokens(&self, ty_name: &syn::Type) -> TokenStream {
        let (name, mut ty, ident) = (&self.name, self.ty.clone(), &self.ident);

        // REVIEW: it is unclear that the `payload` variable contains the JSON string that is Deserialized into the arugment type
        // TODO: there should only ever be 1 argument (+ optional &self)
        let args = self
            .arguments
            .as_ref()
            .unwrap()
            .iter()
            .map(|arg| {
                quote! {
                    ::serde_json::from_str::<#arg>(&payload).unwrap() // TODO: handle error
                }
            });

        // REVIEW: it is super unlear in that `module` is the variable name used in the caller's generated code
        let rcv = self.has_receiver.then(|| {
            quote! { module, }
        });

        quote! {
             #name => {
                 // TODO: handle optional return types from this
                 #ty_name::#ident(#rcv #( #args ),*);
            }
        }
    }

    #[must_use]
    pub fn is_reported(&self) -> bool {
        self.access == AccessType::Reported
    }

    #[must_use]
    pub fn is_desired(&self) -> bool {
        self.access == AccessType::Desired
    }
}

// pub(crate) enum Definition {
//     Reported(reported::Definition),
//     Desired(desired::Definition),
// }

// mod reported {
//     use proc_macro2::TokenStream;
//     use quote::quote;

//     pub(crate) struct Definition {
//         pub(crate) name: String,

//         /// The type of the property that is returned by the resolver
//         pub(crate) ty: syn::Type,
//     }

//     impl Definition {
//         #[must_use]
//         pub(crate) fn method_resolve_prop_tokens(&self) -> TokenStream {
//             let name = &self.name;
//             quote! {
//                 #name => {
//                     // TODO:
//                     println!("reported {}", #name);
//                 }
//             }
//         }
//     }
// }

// mod desired {
//     use proc_macro2::TokenStream;
//     use quote::quote;

//     pub(crate) struct Definition {
//         pub(crate) name: String,

//         /// The type of the property that is set by the resolver
//         pub(crate) ty: syn::Type,

//         // The type of the property that is returned by the resolver
//         // pub(crate) ty: syn::Type,
//     }

//     impl Definition {
//         #[must_use]
//         pub(crate) fn method_resolve_prop_tokens(&self) -> TokenStream {
//             let name = &self.name;
//             quote! {
//                 // TODO: move this into the property::Definition impl
//                 // let ty = &prop.ty;
//                 // let ident = &prop.ident;
//                 // let has_receiver = prop.has_receiver;
//                 // let arguments = prop.arguments.as_ref().unwrap_or(&Vec::new());

//                 #name => {
//                     // TODO:
//                     println!("desired {}", #name);
//                 }
//             }
//         }
//     }
// }

// impl Definition {
//     #[must_use]
//     pub(crate) fn method_resolve_prop_tokens(&self) -> TokenStream {
//         match self {
//             Definition::Reported(def) => def.method_resolve_prop_tokens(),
//             Definition::Desired(def) => def.method_resolve_prop_tokens(),
//         }
//     }

//     #[must_use]
//     pub(crate) fn name(&self) -> &str {
//         match self {
//             Definition::Reported(def) => &def.name,
//             Definition::Desired(def) => &def.name,
//         }
//     }
// }

/// Checks whether all [module properties][1] have different names
#[must_use]
pub fn all_different(props: &[Definition]) -> bool {
    let mut names: Vec<_> = props.iter().map(|def| &def.name).collect();
    names.dedup();
    names.len() == props.len()
}
