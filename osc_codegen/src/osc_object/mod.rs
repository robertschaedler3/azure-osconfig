// use quote::ToTokens;

#[derive(Clone)]
pub(crate) struct Definition {
    /// Method return type
    // pub(crate) ty: syn::Type,

    ///
    pub(crate) name: String,

    ///
    pub(crate) ident: syn::Ident,

    ///
    pub(crate) kind: Kind,

    // TODO:
    // is_async: bool,
}

#[derive(Clone, PartialEq)]
pub(crate) enum Kind {
    Reported,
    Desired,
}