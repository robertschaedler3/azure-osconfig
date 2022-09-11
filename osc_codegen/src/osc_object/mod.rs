// pub mod attr;

use proc_macro2::Span;
use syn::{
    ext::IdentExt as _,
    parse::{Parse, ParseStream},
    token,
};

use crate::common::filter_attrs;

#[derive(Debug, Default)]
pub(crate) struct Attr {
    /// Explictly specified name of this object.
    ///
    /// If [`None`] then the name will be derived from the type name.
    pub(crate) name: Option<String>,

    /// The kind of this object
    pub(crate) kind: Kind,
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
                "reported" | "desired" => {
                    // TODO: check the arguments for the kind
                    // (nice errors for reported, type conversion/deserialization for desired)

                    // TODO: there has to be a better way to do this
                    let _ = input.parse::<token::Comma>();
                    out.kind = match ident.to_string().as_str() {
                        "reported" => Kind::Reported,
                        "desired" => Kind::Desired,
                        _ => unreachable!(),
                    };
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
        // if self.kind != another.kind {
        //     return Err(syn::Error::new(
        //         Span::call_site(),
        //         "duplicate `kind` attribute",
        //     ));
        // }
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

#[derive(Clone)]
pub(crate) struct Definition {
    /// Method return type
    pub(crate) ty: syn::Type,

    ///
    pub(crate) name: String,

    ///
    pub(crate) ident: syn::Ident,

    ///
    pub(crate) kind: Kind,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) enum Kind {
    #[default]
    Reported,
    Desired,
}
