use std::fmt;

use proc_macro2::Span;
use proc_macro_error::{Diagnostic, Level};

pub(crate) const SPEC_URL: &str = "TODO: GitHub URL";

pub(crate) enum Scope {
    // ModuleAttr,
    // ModuleDerive,
    ComponentAttr,
    // ComponentDerive,
    // ObjectAttr,
    // ObjectDerive,
}

impl Scope {
    pub(crate) fn spec_section(&self) -> &str {
        match self {
            Self::ComponentAttr => "TODO: component section",
            // Self::ModuleAttr | Self::ModuleDerive => "TODO: module section",
            // Self::ObjectAttr | Self::ObjectDerive => "TODO: object section"
        }
    }
}

impl fmt::Display for Scope {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let name = match self {
            Self::ComponentAttr => "component",
            // Self::ModuleAttr | Self::ModuleDerive => "module",
            // Self::ComponentAttr | Self::ComponentDerive => "component",
            // Self::ObjectAttr | Self::ObjectDerive => "object",
        };
        write!(f, "OSConfig {name}")
    }
}

impl Scope {
    fn spec_link(&self) -> String {
        format!("{SPEC_URL}{}", self.spec_section())
    }

    pub(crate) fn custom<S: AsRef<str>>(&self, span: Span, msg: S) -> Diagnostic {
        Diagnostic::spanned(span, Level::Error, format!("{self} {}", msg.as_ref()))
            .note(self.spec_link())
    }

    // pub(crate) fn error(&self, err: syn::Error) -> Diagnostic {
    //     Diagnostic::spanned(err.span(), Level::Error, format!("{self} {err}"))
    //         .note(self.spec_link())
    // }

    pub(crate) fn emit_custom<S: AsRef<str>>(&self, span: Span, msg: S) {
        self.custom(span, msg).emit()
    }

    pub(crate) fn custom_error<S: AsRef<str>>(&self, span: Span, msg: S) -> syn::Error {
        syn::Error::new(span, format!("{self} {}", msg.as_ref()))
    }
}
