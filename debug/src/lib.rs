use proc_macro::TokenStream;
use proc_macro2;
use quote::quote;
use syn::{parse_macro_input, parse_quote, DeriveInput};

#[proc_macro_derive(CustomDebug, attributes(debug))]
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

    let mut phan_inner_types: Vec<_> = Vec::new();
    let mut field_types: Vec<_> = Vec::new();
    for f in fields.iter() {
        if let Some(s) = get_phantomdata_inner_type(f)? {
            phan_inner_types.push(s);
        }
        if let Some(s) = get_field_type_name(f)? {
            field_types.push(s);
        }
    }

    let mut fmt_debug_body = proc_macro2::TokenStream::new();
    for field in fields.iter() {
        let field_ident = &field.ident;
        match get_debug_attr_name(field)? {
            Some(s) => fmt_debug_body.extend(quote!(
            .field(stringify!(#field_ident), &format_args!(#s, self.#field_ident))
            )),
            None => fmt_debug_body.extend(quote!(
            .field(stringify!(#field_ident), &self.#field_ident)
            )),
        }
    }

    let mut generics_param = st.generics.clone();
    for g in generics_param.params.iter_mut() {
        if let syn::GenericParam::Type(syn::TypeParam { ref ident, .. }) = g {
            let ident_string = ident.to_string();
            if phan_inner_types.contains(&ident_string) && !field_types.contains(&ident_string) {
                continue;
            }
            if let syn::GenericParam::Type(t) = g {
                t.bounds.push(parse_quote!(std::fmt::Debug));
            }
        }
    }
    let (impl_generics, ty_generics, where_clause) = generics_param.split_for_impl();

    let ret = quote!(
    impl #impl_generics std::fmt::Debug for #struct_ident #ty_generics #where_clause {
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

fn get_debug_attr_name(field: &syn::Field) -> syn::Result<Option<String>> {
    for attr in field.attrs.iter() {
        if let Ok(syn::Meta::NameValue(syn::MetaNameValue {
            ref path, ref lit, ..
        })) = attr.parse_meta()
        {
            if let Some(p) = path.segments.first() {
                if p.ident == "debug" {
                    if let syn::Lit::Str(ref lit_str) = lit {
                        return Ok(Some(lit_str.value()));
                    }
                } else {
                    return Err(syn::Error::new_spanned(p, r#"expected `debug = ""`"#));
                }
            }
        }
    }
    Ok(None)
}

fn get_phantomdata_inner_type(field: &syn::Field) -> syn::Result<Option<String>> {
    if let syn::Type::Path(syn::TypePath {
        path: syn::Path { ref segments, .. },
        ..
    }) = field.ty
    {
        if let Some(syn::PathSegment {
            ref ident,
            ref arguments,
        }) = segments.last()
        {
            if ident == "PhantomData" {
                if let syn::PathArguments::AngleBracketed(syn::AngleBracketedGenericArguments {
                    args,
                    ..
                }) = arguments
                {
                    if let Some(syn::GenericArgument::Type(syn::Type::Path(ref gp))) = args.first()
                    {
                        if let Some(generic_ident) = gp.path.segments.first() {
                            return Ok(Some(generic_ident.ident.clone().to_string()));
                        }
                    }
                }
            }
        }
    }
    Ok(None)
}

fn get_field_type_name(field: &syn::Field) -> syn::Result<Option<String>> {
    if let syn::Type::Path(syn::TypePath {
        path: syn::Path { ref segments, .. },
        ..
    }) = field.ty
    {
        if let Some(syn::PathSegment { ref ident, .. }) = segments.last() {
            return Ok(Some(ident.to_string()));
        }
    }
    Ok(None)
}
