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

    let cursor = syn::buffer::TokenBuffer::new2(st.body.clone());
    let (expand, f) = st.expand_repeat(cursor.begin());
    if f {
        return expand.into();
    }

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

    fn expand_repeat(
        &self,
        origin_cursor: syn::buffer::Cursor,
    ) -> (proc_macro2::TokenStream, bool) {
        let mut ret = proc_macro2::TokenStream::new();
        let mut found = false;

        let mut cursor = origin_cursor;
        while !cursor.eof() {
            if let Some((punct, next_c)) = cursor.punct() {
                if punct.as_char() == '#' {
                    if let Some((group_cursor, _, group_next_cursor)) =
                        next_c.group(proc_macro2::Delimiter::Parenthesis)
                    {
                        if let Some((suffix, suffix_next_cursor)) = group_next_cursor.punct() {
                            if suffix.as_char() == '*' {
                                for i in self.start..self.end {
                                    let t = self.build(&group_cursor.token_stream(), i);
                                    ret.extend(t);
                                }
                                cursor = suffix_next_cursor;
                                found = true;
                                continue;
                            }
                        }
                    }
                }
            }

            if let Some((group_cur, _, next_cur)) = cursor.group(proc_macro2::Delimiter::Brace) {
                let (t, f) = self.expand_repeat(group_cur);
                found = f;
                ret.extend(quote::quote!({#t}));
                cursor = next_cur;
                continue;
            } else if let Some((group_cur, _, next_cur)) =
                cursor.group(proc_macro2::Delimiter::Bracket)
            {
                let (t, f) = self.expand_repeat(group_cur);
                found = f;
                ret.extend(quote::quote!([#t]));
                cursor = next_cur;
                continue;
            } else if let Some((group_cur, _, next_cur)) =
                cursor.group(proc_macro2::Delimiter::Parenthesis)
            {
                let (t, f) = self.expand_repeat(group_cur);
                found = f;
                ret.extend(quote::quote!((#t)));
                cursor = next_cur;
                continue;
            } else if let Some((punct, next_cur)) = cursor.punct() {
                ret.extend(quote::quote!(#punct));
                cursor = next_cur;
                continue;
            } else if let Some((ident, next_cur)) = cursor.ident() {
                ret.extend(quote::quote!(#ident));
                cursor = next_cur;
                continue;
            } else if let Some((literal, next_cur)) = cursor.literal() {
                ret.extend(quote::quote!(#literal));
                cursor = next_cur;
                continue;
            } else if let Some((lifetime, next_cur)) = cursor.lifetime() {
                ret.extend(quote::quote!(#lifetime));
                cursor = next_cur;
                continue;
            }
        }

        (ret, found)
    }
}
