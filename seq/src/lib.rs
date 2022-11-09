use proc_macro::TokenStream;
use proc_macro2;
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input, Ident, LitInt, Token,
};

struct SeqParse {
    ident: Ident,
    start: isize,
    end: isize,
    body: proc_macro2::TokenStream,
}

impl Parse for SeqParse {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // syn::Ident, Token![in], syn::LitInt, Token![..], syn::LitInt.
        let ident: Ident = input.parse()?;
        input.parse::<Token![in]>()?;
        let start: LitInt = input.parse()?;
        input.parse::<Token![..]>()?;
        let end: LitInt = input.parse()?;
        let body_buf;
        syn::braced!(body_buf in input);
        let body: proc_macro2::TokenStream = body_buf.parse()?;
        Ok(SeqParse {
            ident,
            start: start.base10_parse()?,
            end: end.base10_parse()?,
            body,
        })
    }
}

#[proc_macro]
pub fn seq(input: TokenStream) -> TokenStream {
    let st = parse_macro_input!(input as SeqParse);
    TokenStream::new()
}
