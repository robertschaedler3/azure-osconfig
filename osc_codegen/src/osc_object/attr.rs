use proc_macro2::TokenStream;

pub fn expand(_attr_args: TokenStream, body: TokenStream) -> syn::Result<TokenStream> {
    Ok(body)
}