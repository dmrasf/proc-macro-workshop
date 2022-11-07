use proc_macro::TokenStream;
use proc_macro2;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(CustomDebug)]
pub fn derive(input: TokenStream) -> TokenStream {
    let st = parse_macro_input!(input as DeriveInput);
    match expand(&st) {
        Ok(token) => token.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

fn expand(st: &DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let struct_ident = &st.ident;
    let fields = get_struct_fields(st)?;

    let mut fmt_debug_body = proc_macro2::TokenStream::new();
    for field in fields.iter() {
        let field_ident = &field.ident;
        fmt_debug_body.extend(quote!(
            .field(stringify!(#field_ident), &self.#field_ident)
        ));
    }

    let ret = quote!(
        impl std::fmt::Debug for #struct_ident {
            fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
                fmt.debug_struct(stringify!(#struct_ident))
                    #fmt_debug_body
                    .finish()
            }
        }
    );
    Ok(ret)
}

fn get_struct_fields(
    st: &DeriveInput,
) -> syn::Result<&syn::punctuated::Punctuated<syn::Field, syn::Token![,]>> {
    if let syn::Data::Struct(syn::DataStruct {
        fields: syn::Fields::Named(syn::FieldsNamed { ref named, .. }),
        ..
    }) = st.data
    {
        return Ok(named);
    }
    Err(syn::Error::new_spanned(st, "miss field"))
}
