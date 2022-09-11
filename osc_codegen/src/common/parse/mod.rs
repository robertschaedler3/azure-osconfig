pub(crate) mod attr;

/// Extension of [`syn::Type`] providing common function widely used by this crate for parsing.
pub(crate) trait TypeExt {
    /// Retrieves the innermost non-parenthesized [`syn::Type`] from the given
    /// one (unwraps nested [`syn::TypeParen`]s asap).
    #[must_use]
    fn unparenthesized(&self) -> &Self;

    // /// Retrieves the inner [`syn::Type`] from the given reference type, or just
    // /// returns "as is" if the type is not a reference.
    // ///
    // /// Also, makes the type [`TypeExt::unparenthesized`], if possible.
    // #[must_use]
    // fn unreferenced(&self) -> &Self;

    // /// Iterates mutably over all the lifetime parameters of this [`syn::Type`]
    // /// with the given `func`tion.
    // fn lifetimes_iter_mut<F: FnMut(&mut syn::Lifetime)>(&mut self, func: &mut F);

    // /// Anonymizes all the lifetime parameters of this [`syn::Type`] (except
    // /// the `'static` ones), making it suitable for using in contexts with
    // /// inferring.
    // fn lifetimes_anonymized(&mut self);

    /// Returns the topmost [`syn::Ident`] of this [`syn::TypePath`], if any.
    #[must_use]
    fn topmost_ident(&self) -> Option<&syn::Ident>;
}

impl TypeExt for syn::Type {
    fn unparenthesized(&self) -> &Self {
        match self {
            Self::Paren(ty) => ty.elem.unparenthesized(),
            Self::Group(ty) => ty.elem.unparenthesized(),
            ty => ty,
        }
    }

    fn topmost_ident(&self) -> Option<&syn::Ident> {
        match self.unparenthesized() {
            syn::Type::Path(p) => Some(&p.path),
            syn::Type::Reference(r) => match (*r.elem).unparenthesized() {
                syn::Type::Path(p) => Some(&p.path),
                syn::Type::TraitObject(o) => match o.bounds.iter().next().unwrap() {
                    syn::TypeParamBound::Trait(b) => Some(&b.path),
                    _ => None,
                },
                _ => None,
            },
            _ => None,
        }?
        .segments
        .last()
        .map(|s| &s.ident)
    }
}