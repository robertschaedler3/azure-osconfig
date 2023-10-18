use std::fmt;

use proc_macro2::Span;
use proc_macro_error::{Diagnostic, Level};

pub(crate) const SPEC_URL: &str = "TODO:";

pub(crate) enum Scope {
    ObjectAttr,
}

impl Scope {
    pub(crate) fn spec_section(&self) -> &str {
        match self {
            Self::ObjectAttr => "TODO: #spec-section",
        }
    }
}

impl fmt::Display for Scope {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let name = match self {
            Self::ObjectAttr => "object",
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

    pub(crate) fn emit_custom<S: AsRef<str>>(&self, span: Span, msg: S) {
        self.custom(span, msg).emit()
    }

    pub(crate) fn custom_error<S: AsRef<str>>(&self, span: Span, msg: S) -> syn::Error {
        syn::Error::new(span, format!("{self} {}", msg.as_ref()))
    }
}