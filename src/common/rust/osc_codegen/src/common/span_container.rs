use std::{
    hash::{Hash, Hasher},
    ops,
};

use proc_macro2::{Span, TokenStream};
use quote::ToTokens;

#[derive(Clone, Copy, Debug)]
pub(crate) struct SpanContainer<T> {
    _expr: Option<Span>, // REVIEW:
    ident: Span,
    val: T,
}

impl<T: ToTokens> ToTokens for SpanContainer<T> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.val.to_tokens(tokens)
    }
}

impl<T> SpanContainer<T> {
    pub(crate) fn new(ident: Span, _expr: Option<Span>, val: T) -> Self {
        Self { _expr, ident, val }
    }

    pub(crate) fn span_ident(&self) -> Span {
        self.ident
    }

    pub(crate) fn into_inner(self) -> T {
        self.val
    }
}

impl<T> AsRef<T> for SpanContainer<T> {
    fn as_ref(&self) -> &T {
        &self.val
    }
}

impl<T> ops::Deref for SpanContainer<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.val
    }
}

impl<T: PartialEq> PartialEq for SpanContainer<T> {
    fn eq(&self, other: &Self) -> bool {
        self.val == other.val
    }
}

impl<T: Eq> Eq for SpanContainer<T> {}

impl<T: PartialEq> PartialEq<T> for SpanContainer<T> {
    fn eq(&self, other: &T) -> bool {
        &self.val == other
    }
}

impl<T: Hash> Hash for SpanContainer<T> {
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        self.val.hash(state)
    }
}