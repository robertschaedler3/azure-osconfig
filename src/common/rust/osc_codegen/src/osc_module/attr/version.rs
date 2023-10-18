use quote::ToTokens;
use syn::parse::{Parse, ParseStream};

// use core::module::Version;

#[derive(Default, Debug, Eq, PartialEq)]
pub(crate) struct Version {
    pub(crate) major: u32,
    pub(crate) minor: u32,
    pub(crate) patch: u32,
    pub(crate) tweak: u32,
}

impl Parse for Version {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let version = input.parse::<syn::LitStr>()?;
        let version = version.value();
        let mut version = version.split('.'); // TODO: validate string

        let major = version.next().unwrap_or("0").parse().unwrap_or(0);
        let minor = version.next().unwrap_or("0").parse().unwrap_or(0);
        let patch = version.next().unwrap_or("0").parse().unwrap_or(0);
        let tweak = version.next().unwrap_or("0").parse().unwrap_or(0);

        Ok(Self {
            major,
            minor,
            patch,
            tweak,
        })
    }
}

impl ToTokens for Version {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let major = self.major;
        let minor = self.minor;
        let patch = self.patch;
        let tweak = self.tweak;

        tokens.extend(quote::quote! {
            ::osc::module::Version {
                major: #major,
                minor: #minor,
                patch: #patch,
                tweak: #tweak,
            }
        });
    }
}
