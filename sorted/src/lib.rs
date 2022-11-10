use proc_macro::TokenStream;
use syn::{parse_macro_input, Item};

#[proc_macro_attribute]
pub fn sorted(args: TokenStream, input: TokenStream) -> TokenStream {
    let st = parse_macro_input!(input as Item);

    match expand(&st) {
        Ok(ts) => ts.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

fn expand(st: &Item) -> syn::Result<proc_macro2::TokenStream> {
    let ret = proc_macro2::TokenStream::new();
    Ok(ret)
}
