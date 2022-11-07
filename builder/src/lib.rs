use proc_macro::TokenStream;
use proc_macro2;
use quote::quote;
use syn::{parse_macro_input, spanned::Spanned, DeriveInput};

#[proc_macro_derive(Builder)]
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
    let builder_struct_ident =
        syn::Ident::new(&format!("{}Builder", struct_ident.to_string()), st.span());

    let mut builder_struct_content = proc_macro2::TokenStream::new();
    let mut builder_fn_content = proc_macro2::TokenStream::new();
    let mut builder_setters = proc_macro2::TokenStream::new();
    let mut check_field_is_none = proc_macro2::TokenStream::new();
    let mut builder_to_struct_content = proc_macro2::TokenStream::new();
    for f in fields.iter() {
        let ident = &f.ident;
        let ty = &f.ty;
        builder_struct_content.extend(quote!(
        #ident: std::option::Option<#ty>,
        ));
        builder_fn_content.extend(quote!(
        #ident: std::option::Option::None,
        ));
        builder_setters.extend(quote!(
        fn #ident(&mut self, #ident: #ty) -> &mut Self {
            self.#ident = Some(#ident);
            self
        }
        ));
        check_field_is_none.extend(quote!(
        if self.#ident.is_none() {
            let err = format!("{} is None", stringify!(#ident));
            return Err(err.into());
        }
        ));
        builder_to_struct_content.extend(quote::quote!(
        #ident: self.#ident.clone().unwrap(),
        ));
    }

    let ret = quote!(
    pub struct #builder_struct_ident {
        #builder_struct_content
    }
    impl #struct_ident {
        pub fn builder() -> #builder_struct_ident {
            #builder_struct_ident {
                #builder_fn_content
            }
        }
    }
    impl #builder_struct_ident {
        #builder_setters
        pub fn build(
            &mut self
        ) -> std::result::Result<#struct_ident, std::boxed::Box<dyn std::error::Error>> {
            #check_field_is_none
            Ok(#struct_ident {
                #builder_to_struct_content
            })
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
