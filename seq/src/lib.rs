use proc_macro::TokenStream;
use proc_macro2::{self, TokenTree};
use quote::quote;
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

    let mut ret = proc_macro2::TokenStream::new();
    for i in st.start..st.end {
        ret.extend(st.build(&st.body, i));
    }
    ret.into()
}

impl SeqParse {
    fn build(&self, bd: &proc_macro2::TokenStream, i: isize) -> proc_macro2::TokenStream {
        let buf = bd.clone().into_iter().collect::<Vec<_>>();
        let mut ret = proc_macro2::TokenStream::new();

        let mut idx = 0usize;
        while idx < buf.len() {
            let item = &buf[idx];
            match item {
                TokenTree::Group(group) => {
                    let new_stream = self.build(&group.stream(), i);
                    let mut g = proc_macro2::Group::new(group.delimiter(), new_stream);
                    g.set_span(group.span().clone());
                    ret.extend(quote!(#g));
                }
                TokenTree::Ident(ident) => {
                    if idx + 2 < buf.len() {
                        if let TokenTree::Punct(p) = &buf[idx + 1] {
                            if p.as_char() == '~' {
                                if let TokenTree::Ident(_ident) = &buf[idx + 2] {
                                    if _ident == &self.ident {
                                        let new_ident_litral =
                                            format!("{}{}", ident.to_string(), i);
                                        let new_ident = proc_macro2::Ident::new(
                                            new_ident_litral.as_str(),
                                            ident.span(),
                                        );
                                        ret.extend(quote::quote!(#new_ident));
                                        idx += 3;
                                        continue;
                                    }
                                }
                            }
                        }
                    }
                    if ident == &self.ident {
                        let new_ident = proc_macro2::Literal::i64_unsuffixed(i as i64);
                        ret.extend(quote!(#new_ident));
                    } else {
                        ret.extend(quote!(#item));
                    }
                }
                _ => ret.extend(quote!(#item)),
            }
            idx += 1;
        }
        ret
    }
}
