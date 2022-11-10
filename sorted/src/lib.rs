use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::ToTokens;
use syn::{
    parse_macro_input,
    visit_mut::{self, VisitMut},
    Item, ItemEnum, ItemFn,
};

#[proc_macro_attribute]
pub fn sorted(_args: TokenStream, input: TokenStream) -> TokenStream {
    let st = parse_macro_input!(input as Item);

    match expand(&st) {
        Ok(ts) => ts.into(),
        Err(e) => {
            let mut ts = e.to_compile_error();
            ts.extend(st.to_token_stream());
            ts.into()
        }
    }
}

fn expand(st: &Item) -> syn::Result<proc_macro2::TokenStream> {
    if let Item::Enum(eu) = st {
        return check_order(eu);
    } else {
        syn::Result::Err(syn::Error::new(
            Span::call_site(),
            "expected enum or match expression",
        ))
    }
}

fn check_order(eu: &ItemEnum) -> syn::Result<proc_macro2::TokenStream> {
    let variant_names: Vec<_> = eu.variants.iter().map(|item| &item.ident).collect();

    let mut sorted_variant_names = variant_names.clone();
    sorted_variant_names.sort_by(|a, b| a.to_string().cmp(&b.to_string()));

    for (ori, sorted) in variant_names.iter().zip(sorted_variant_names.iter()) {
        if ori != sorted {
            return syn::Result::Err(syn::Error::new(
                sorted.span(),
                format!(
                    "{} should sort before {}",
                    sorted.to_string(),
                    ori.to_string()
                ),
            ));
        }
    }

    Ok(eu.to_token_stream())
}

#[proc_macro_attribute]
pub fn check(_args: TokenStream, input: TokenStream) -> TokenStream {
    let mut st = parse_macro_input!(input as ItemFn);

    match match_expand(&mut st) {
        Ok(ts) => ts.into(),
        Err(e) => {
            let mut ts = e.to_compile_error();
            ts.extend(st.to_token_stream());
            ts.into()
        }
    }
}

struct MatchVisitor {
    err: Option<syn::Error>,
}
impl VisitMut for MatchVisitor {
    fn visit_expr_match_mut(&mut self, i: &mut syn::ExprMatch) {
        let mut remove_idx: isize = -1;
        for (idx, attr) in i.attrs.iter().enumerate() {
            if let Some(ps) = attr.path.segments.first() {
                if ps.ident.to_string() == "sorted" {
                    remove_idx = idx as isize;
                    break;
                }
            }
        }

        if remove_idx != -1 {
            i.attrs.remove(remove_idx as usize);

            let mut ori_idents = Vec::new();
            for arm in &i.arms {
                if let syn::Pat::TupleStruct(syn::PatTupleStruct { ref path, .. }) = arm.pat {
                    if let Some(ps) = path.segments.first() {
                        ori_idents.push(&ps.ident);
                    }
                }
            }

            let mut sorted_idents = ori_idents.clone();
            sorted_idents.sort_by(|a, b| a.to_string().cmp(&b.to_string()));

            for (ori, sorted) in ori_idents.iter().zip(sorted_idents.iter()) {
                if ori != sorted {
                    self.err = Some(syn::Error::new_spanned(
                        sorted.to_token_stream(),
                        format!(
                            "{} should sort before {}",
                            sorted.to_string(),
                            ori.to_string()
                        ),
                    ));
                    return;
                }
            }
        }

        visit_mut::visit_expr_match_mut(self, i);
    }
}

fn match_expand(st: &mut ItemFn) -> syn::Result<proc_macro2::TokenStream> {
    let mut visitor = MatchVisitor { err: None };
    visitor.visit_item_fn_mut(st);

    if visitor.err.is_none() {
        syn::Result::Ok(st.to_token_stream())
    } else {
        syn::Result::Err(visitor.err.unwrap())
    }
}
