use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::ToTokens;
use syn::{parse_macro_input, Item, ItemEnum};

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
