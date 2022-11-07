use proc_macro::TokenStream;
use proc_macro2;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(Builder)]
pub fn derive(input: TokenStream) -> TokenStream {
    let st = parse_macro_input!(input as DeriveInput);
    match expand(&st) {
        Ok(token) => token.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

fn expand(st: &DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let ret = proc_macro2::TokenStream::new();
    Ok(ret)
}
