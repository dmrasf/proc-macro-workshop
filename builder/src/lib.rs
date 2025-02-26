use proc_macro::TokenStream;
use proc_macro2;
use quote::quote;
use syn::{parse_macro_input, spanned::Spanned, DeriveInput};

#[proc_macro_derive(Builder, attributes(builder))]
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
        let inner_ty = match get_field_inner_type(ty, "Option".to_string()) {
            Some(inner_ty) => inner_ty,
            None => ty,
        };
        let inner_vec_ty = match get_field_inner_type(inner_ty, "Vec".to_string()) {
            Some(inner_vec_ty) => inner_vec_ty,
            None => inner_ty,
        };
        let attr = get_attr_builder_name(f)?;

        builder_fn_content.extend(quote!(
        #ident: std::option::Option::None,
        ));
        builder_struct_content.extend(quote!(
        #ident: std::option::Option<#inner_ty>,
        ));
        if inner_ty == inner_vec_ty || attr.is_none() {
            builder_setters.extend(quote!(
            fn #ident(&mut self, #ident: #inner_ty) -> &mut Self {
                self.#ident = Some(#ident);
                self
            }
            ));
        } else {
            let attr_ident = &attr.unwrap();
            if ident.as_ref() != Some(attr_ident) {
                builder_setters.extend(quote!(
                fn #ident(&mut self, #ident: #inner_ty) -> &mut Self {
                    self.#ident = Some(#ident);
                    self
                }
                ));
            }
            builder_setters.extend(quote!(
            fn #attr_ident(&mut self, #attr_ident: #inner_vec_ty) -> &mut Self {
                if let Some(ref mut v) = self.#ident {
                    v.push(#attr_ident);
                } else {
                    self.#ident = std::option::Option::Some(vec![#attr_ident]);
                }
                self
            }
            ));
        }
        if ty == inner_ty {
            builder_to_struct_content.extend(quote::quote!(
            #ident: self.#ident.clone().unwrap(),
            ));
            if inner_ty != inner_vec_ty {
                check_field_is_none.extend(quote!(
                if self.#ident.is_none() {
                    self.#ident = std::option::Option::Some(vec![]);
                }
                ));
            } else {
                check_field_is_none.extend(quote!(
                if self.#ident.is_none() {
                    let err = format!("{} is None", stringify!(#ident));
                    return Err(err.into());
                }
                ));
            }
        } else {
            builder_to_struct_content.extend(quote::quote!(
            #ident: self.#ident.clone(),
            ));
        }
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

fn get_field_inner_type(ty: &syn::Type, s: String) -> Option<&syn::Type> {
    if let syn::Type::Path(syn::TypePath {
        path: syn::Path { ref segments, .. },
        ..
    }) = ty
    {
        if let Some(seg) = segments.last() {
            if seg.ident == s {
                if let syn::PathArguments::AngleBracketed(syn::AngleBracketedGenericArguments {
                    ref args,
                    ..
                }) = seg.arguments
                {
                    if let Some(syn::GenericArgument::Type(inner_ty)) = args.first() {
                        return Some(inner_ty);
                    }
                }
            }
        }
    }
    None
}

fn get_attr_builder_name(field: &syn::Field) -> syn::Result<Option<syn::Ident>> {
    for attr in &field.attrs {
        if let Ok(syn::Meta::List(syn::MetaList {
            ref path,
            ref nested,
            ..
        })) = attr.parse_meta()
        {
            if let Some(p) = path.segments.first() {
                if p.ident == "builder" {
                    if let Some(syn::NestedMeta::Meta(syn::Meta::NameValue(kv))) = nested.first() {
                        if kv.path.is_ident("each") {
                            if let syn::Lit::Str(ref ident_str) = kv.lit {
                                return Ok(Some(syn::Ident::new(
                                    ident_str.value().as_str(),
                                    attr.span(),
                                )));
                            }
                        } else {
                            if let Ok(syn::Meta::List(ref list)) = attr.parse_meta() {
                                return Err(syn::Error::new_spanned(
                                    list,
                                    r#"expected `builder(each = "...")`"#,
                                ));
                            }
                        }
                    }
                }
            }
        }
    }
    Ok(None)
}
