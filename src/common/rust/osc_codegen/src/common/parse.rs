//! Common functions, definitions and extensions for parsing, normalizing and modifying Rust syntax,
//! used by this crate.

pub(crate) mod attr;

use std::{any::TypeId, iter};

use proc_macro2::Span;
use syn::{
    ext::IdentExt as _,
    parse::{Parse, ParseBuffer},
    punctuated::Punctuated,
    token::{self, Token},
};

/// Extension of [`ParseBuffer`] providing common function widely used by this crate for parsing.
pub(crate) trait ParseBufferExt {
    /// Tries to parse `T` as the next token.
    ///
    /// Doesn't move [`ParseStream`]'s cursor if there is no `T`.
    fn try_parse<T: Default + Parse + Token>(&self) -> syn::Result<Option<T>>;

    /// Checks whether next token is `T`.
    ///
    /// Doesn't move [`ParseStream`]'s cursor.
    #[must_use]
    fn is_next<T: Default + Token>(&self) -> bool;

    /// Parses next token as [`syn::Ident`] _allowing_ Rust keywords, while default [`Parse`]
    /// implementation for [`syn::Ident`] disallows keywords.
    ///
    /// Always moves [`ParseStream`]'s cursor.
    fn parse_any_ident(&self) -> syn::Result<syn::Ident>;
}

impl<'a> ParseBufferExt for ParseBuffer<'a> {
    fn try_parse<T: Default + Parse + Token>(&self) -> syn::Result<Option<T>> {
        Ok(if self.is_next::<T>() {
            Some(self.parse()?)
        } else {
            None
        })
    }

    fn is_next<T: Default + Token>(&self) -> bool {
        self.lookahead1().peek(|_| T::default())
    }

    fn parse_any_ident(&self) -> syn::Result<syn::Ident> {
        self.call(syn::Ident::parse_any)
    }
}

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

    // fn unreferenced(&self) -> &Self {
    //     match self.unparenthesized() {
    //         Self::Reference(ref_ty) => &ref_ty.elem,
    //         ty => ty,
    //     }
    // }

    // fn lifetimes_iter_mut<F: FnMut(&mut syn::Lifetime)>(&mut self, func: &mut F) {
    //     use syn::{GenericArgument as GA, Type as T};

    //     fn iter_path<F: FnMut(&mut syn::Lifetime)>(path: &mut syn::Path, func: &mut F) {
    //         for seg in path.segments.iter_mut() {
    //             match &mut seg.arguments {
    //                 syn::PathArguments::AngleBracketed(angle) => {
    //                     for arg in angle.args.iter_mut() {
    //                         match arg {
    //                             GA::Lifetime(lt) => func(lt),
    //                             GA::Type(ty) => ty.lifetimes_iter_mut(func),
    //                             GA::Binding(b) => b.ty.lifetimes_iter_mut(func),
    //                             GA::Constraint(_) | GA::Const(_) => {}
    //                         }
    //                     }
    //                 }
    //                 syn::PathArguments::Parenthesized(args) => {
    //                     for ty in args.inputs.iter_mut() {
    //                         ty.lifetimes_iter_mut(func)
    //                     }
    //                     if let syn::ReturnType::Type(_, ty) = &mut args.output {
    //                         (*ty).lifetimes_iter_mut(func)
    //                     }
    //                 }
    //                 syn::PathArguments::None => {}
    //             }
    //         }
    //     }

    //     match self {
    //         T::Array(syn::TypeArray { elem, .. })
    //         | T::Group(syn::TypeGroup { elem, .. })
    //         | T::Paren(syn::TypeParen { elem, .. })
    //         | T::Ptr(syn::TypePtr { elem, .. })
    //         | T::Slice(syn::TypeSlice { elem, .. }) => (*elem).lifetimes_iter_mut(func),

    //         T::Tuple(syn::TypeTuple { elems, .. }) => {
    //             for ty in elems.iter_mut() {
    //                 ty.lifetimes_iter_mut(func)
    //             }
    //         }

    //         T::ImplTrait(syn::TypeImplTrait { bounds, .. })
    //         | T::TraitObject(syn::TypeTraitObject { bounds, .. }) => {
    //             for bound in bounds.iter_mut() {
    //                 match bound {
    //                     syn::TypeParamBound::Lifetime(lt) => func(lt),
    //                     syn::TypeParamBound::Trait(bound) => {
    //                         if bound.lifetimes.is_some() {
    //                             todo!("Iterating over HRTB lifetimes in trait is not yet supported")
    //                         }
    //                         iter_path(&mut bound.path, func)
    //                     }
    //                 }
    //             }
    //         }

    //         T::Reference(ref_ty) => {
    //             if let Some(lt) = ref_ty.lifetime.as_mut() {
    //                 func(lt)
    //             }
    //             (*ref_ty.elem).lifetimes_iter_mut(func)
    //         }

    //         T::Path(ty) => iter_path(&mut ty.path, func),

    //         // These types unlikely will be used as OSConfig types.
    //         T::BareFn(_) | T::Infer(_) | T::Macro(_) | T::Never(_) | T::Verbatim(_) => {}

    //         // Following the syn idiom for exhaustive matching on Type:
    //         // https://github.com/dtolnay/syn/blob/1.0.90/src/ty.rs#L67-L87
    //         // TODO: #[cfg_attr(test, deny(non_exhaustive_omitted_patterns))]
    //         //       https://github.com/rust-lang/rust/issues/89554
    //         _ => unimplemented!(),
    //     }
    // }

    // fn lifetimes_anonymized(&mut self) {
    //     self.lifetimes_iter_mut(&mut |lt| {
    //         if lt.ident != "_" && lt.ident != "static" {
    //             lt.ident = syn::Ident::new("_", Span::call_site());
    //         }
    //     });
    // }

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
