use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse_macro_input, spanned::Spanned, Attribute, Data, DeriveInput, Error, Fields, Ident, LitStr,
    Type, Variant,
};

/// Derive macro that implements `ToHref` for an enum.
/// Checks for `#[route(path="...")]` or `#[parent_route(path="...")]`.
/// If exactly one unnamed field is present, we treat it as nested.
pub fn derive_to_href_impl(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let Data::Enum(data_enum) = ast.data else {
        return syn::Error::new_spanned(ast, "`#[derive(ToHref)]` can only be used on an enum.")
            .to_compile_error()
            .into();
    };

    let enum_ident = &ast.ident;
    let mut match_arms = Vec::new();
    let mut errors = Vec::new();

    for variant in data_enum.variants {
        match process_variant(enum_ident, variant) {
            Ok(Some(ts)) => match_arms.push(ts),
            Ok(None) => (),
            Err(e) => errors.push(e),
        }
    }

    if !errors.is_empty() {
        let mut combined = proc_macro2::TokenStream::new();
        for e in errors {
            combined.extend(e.to_compile_error());
        }
        return combined.into();
    }

    let fallback_arm = quote! {
        _ => "/".to_string()
    };

    let expanded = quote! {
        impl std::fmt::Display for #enum_ident {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", match self {
                    #( #match_arms, )*
                    #fallback_arm
                })
            }
        }

        impl ::leptos_router::components::ToHref for #enum_ident {
            fn to_href(&self) -> Box<dyn Fn() -> String + '_> {
                let owned_self = self.clone();
                Box::new(move || {
                    match &owned_self {
                        #( #match_arms, )*
                        #fallback_arm
                    }
                })
            }
        }
    };

    expanded.into()
}

fn process_variant(
    enum_ident: &Ident,
    variant: Variant,
) -> syn::Result<Option<proc_macro2::TokenStream>> {
    let Variant { ident, fields, attrs, .. } = variant;

    let route_path = match find_route_path(&attrs) {
        Some(p) if !p.is_empty() => p,
        _ => {
            // If no route attribute but exactly 1 unnamed field => nested
            if let Fields::Unnamed(unnamed) = &fields {
                if unnamed.unnamed.len() == 1 {
                    let pat = quote!( #enum_ident::#ident(nested) );
                    return Ok(Some(quote! {
                        #pat => nested.to_string()
                    }));
                }
            }
            return Ok(None);
        }
    };

    let field_infos = extract_variant_fields(enum_ident, &ident, &fields)?;
    validate_path_and_fields(&route_path, &field_infos, &fields, &ident)?;
    let (variant_pat, fields_for_build) = build_variant_pattern(enum_ident, &ident, &fields)?;
    let build_code = generate_path_builder(&route_path, &fields_for_build);

    // If exactly 1 unnamed field => prefix + nested
    if let Fields::Unnamed(unnamed) = &fields {
        if unnamed.unnamed.len() == 1 {
            return Ok(Some(quote! {
                #variant_pat => {
                    let prefix_str = {
                        #build_code
                    };
                    let nested_str = _0.to_string();
                    combine_paths(&prefix_str, &nested_str)
                }
            }));
        }
    }

    // Normal case
    Ok(Some(quote! {
        #variant_pat => {
            #build_code
        }
    }))
}

struct FieldMeta {
    name: String,
    ty: Type,
    span: proc_macro2::Span,
}

fn extract_variant_fields(
    _enum_ident: &Ident,
    variant_ident: &Ident,
    fields: &Fields,
) -> syn::Result<Vec<FieldMeta>> {
    match fields {
        Fields::Unit => Ok(Vec::new()),
        Fields::Named(named) => {
            let mut out = Vec::new();
            for f in &named.named {
                let ident = f.ident.as_ref().unwrap().clone();
                out.push(FieldMeta {
                    name: ident.to_string(),
                    ty: f.ty.clone(),
                    span: f.span(),
                });
            }
            Ok(out)
        }
        Fields::Unnamed(unnamed) => {
            let count = unnamed.unnamed.len();
            if count == 0 {
                return Err(Error::new(
                    unnamed.span(),
                    format!("Variant `{}` has 0 fields, expected 1 for nested routing.", variant_ident),
                ));
            }
            if count > 1 {
                return Err(Error::new(
                    unnamed.span(),
                    format!("Variant `{}` has {} fields, but only 1 is allowed for nested routing.", variant_ident, count),
                ));
            }
            let only_field = &unnamed.unnamed[0];
            Ok(vec![FieldMeta {
                name: "_0".to_string(),
                ty: only_field.ty.clone(),
                span: only_field.span(),
            }])
        }
    }
}

fn validate_path_and_fields(
    route_str: &str,
    fields: &[FieldMeta],
    syn_fields: &Fields,
    variant_ident: &Ident,
) -> syn::Result<()> {
    let segments = parse_segments(route_str);
    let mut used_fields = Vec::new();

    for seg in &segments {
        match seg {
            RouteSegment::Static(_) => {}
            RouteSegment::Param(name) => {
                used_fields.push(name.clone());
                if !fields.iter().any(|f| f.name == *name) {
                    return Err(Error::new(
                        variant_ident.span(),
                        format!("Path param `:{}` not found in `{}`.", name, variant_ident),
                    ));
                }
            }
            RouteSegment::OptionalParam(name) => {
                used_fields.push(name.clone());
                let Some(field_meta) = fields.iter().find(|f| f.name == *name) else {
                    return Err(Error::new(
                        variant_ident.span(),
                        format!("Optional param `:{}?` not found in `{}`.", name, variant_ident),
                    ));
                };
                if !is_option_type(&field_meta.ty) {
                    return Err(Error::new(
                        field_meta.span,
                        format!("`:{}?` in route requires `Option<T>` field for `{}`.", name, variant_ident),
                    ));
                }
            }
        }
    }

    // Single unnamed => skip leftover check
    if let Fields::Unnamed(unnamed) = syn_fields {
        if unnamed.unnamed.len() == 1 {
            return Ok(());
        }
    }

    // Otherwise leftover fields must be Option<T>
    for f in fields {
        if !used_fields.contains(&f.name) && !is_option_type(&f.ty) {
            return Err(Error::new(
                f.span,
                format!("Field `{}` not used in path, so must be `Option<T>` to appear as a query.", f.name),
            ));
        }
    }

    Ok(())
}

fn build_variant_pattern(
    enum_ident: &Ident,
    variant_ident: &Ident,
    fields: &Fields,
) -> syn::Result<(proc_macro2::TokenStream, Vec<(String, Type)>)> {
    match fields {
        Fields::Unit => {
            let pat = quote!( #enum_ident::#variant_ident );
            Ok((pat, vec![]))
        }
        Fields::Named(named) => {
            let mut field_names = Vec::new();
            let mut field_info = Vec::new();
            for f in &named.named {
                let nm = f.ident.as_ref().unwrap().clone();
                field_names.push(quote!(#nm));
                field_info.push((nm.to_string(), f.ty.clone()));
            }
            let pat = quote!( #enum_ident::#variant_ident { #( #field_names ),* } );
            Ok((pat, field_info))
        }
        Fields::Unnamed(unnamed) => {
            let f = &unnamed.unnamed[0];
            let field_ident = syn::Ident::new("_0", f.span());
            let pat = quote!( #enum_ident::#variant_ident(#field_ident) );
            Ok((pat, vec![("_0".to_string(), f.ty.clone())]))
        }
    }
}

fn generate_path_builder(route: &str, fields: &[(String, Type)]) -> proc_macro2::TokenStream {
    let segments = parse_segments(route);
    let mut used_fields = Vec::new();

    let segment_stmts: Vec<_> = segments
        .into_iter()
        .map(|seg| match seg {
            RouteSegment::Static(txt) => quote! {
                if path.is_empty() {
                    path.push('/');
                } else if !path.ends_with('/') {
                    path.push('/');
                }
                path.push_str(#txt);
            },
            RouteSegment::Param(name) => {
                used_fields.push(name.clone());
                let field_ident = syn::Ident::new(&name, proc_macro2::Span::call_site());
                quote! {
                    path.push('/');
                    path.push_str(&#field_ident.to_string());
                }
            }
            RouteSegment::OptionalParam(name) => {
                used_fields.push(name.clone());
                let field_ident = syn::Ident::new(&name, proc_macro2::Span::call_site());
                quote! {
                    if let Some(ref val) = #field_ident {
                        path.push('/');
                        path.push_str(&val.to_string());
                    }
                }
            }
        })
        .collect();

    let leftover_fields: Vec<_> = fields
        .iter()
        .filter(|(n, _)| !used_fields.contains(n))
        .collect();

    let query_push = leftover_fields.into_iter().map(|(fname, fty)| {
        if is_option_type(fty) {
            let field_ident = syn::Ident::new(fname, proc_macro2::Span::call_site());
            quote! {
                if let Some(ref val) = #field_ident {
                    query_vec.push((#fname.to_owned(), val.to_string()));
                }
            }
        } else {
            quote!()
        }
    });

    quote! {
        let mut path = String::new();
        #(#segment_stmts)*

        let mut query_vec: Vec<(String, String)> = Vec::new();
        #(#query_push)*

        if !query_vec.is_empty() {
            query_vec.sort_by(|a, b| a.0.cmp(&b.0));
            path.push('?');
            let mut first = true;
            for (k, v) in query_vec {
                if !first { path.push('&'); } else { first = false; }
                path.push_str(&k);
                path.push('=');
                path.push_str(&v);
            }
        }

        if path.is_empty() {
            path.push('/');
        }
        path
    }
}

/* ---------------------------------------------------------------------- *
 * SEGMENTS & HELPERS
 * ---------------------------------------------------------------------- */
#[derive(Debug)]
enum RouteSegment {
    Static(String),
    Param(String),
    OptionalParam(String),
}

fn parse_segments(route: &str) -> Vec<RouteSegment> {
    let without_leading = route.trim_start_matches('/');
    let mut segs = Vec::new();
    for part in without_leading.split('/') {
        if part.starts_with(':') {
            if let Some(stripped) = part.strip_suffix('?') {
                segs.push(RouteSegment::OptionalParam(
                    stripped.trim_start_matches(':').to_string(),
                ));
            } else {
                segs.push(RouteSegment::Param(part.trim_start_matches(':').to_string()));
            }
        } else if !part.is_empty() {
            segs.push(RouteSegment::Static(part.to_string()));
        }
    }
    segs
}

fn is_option_type(ty: &Type) -> bool {
    if let syn::Type::Path(tp) = ty {
        if let Some(seg) = tp.path.segments.last() {
            return seg.ident == "Option";
        }
    }
    false
}

fn find_route_path(attrs: &[Attribute]) -> Option<String> {
    for attr in attrs {
        // TODO: Integrate into Routable
        if attr.path().is_ident("route")
            || attr.path().is_ident("parent_route")
            || attr.path().is_ident("protected_route")
            || attr.path().is_ident("protected_parent_route")  {
            let mut path = None;
            let _ = attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("path") {
                    let value = meta.value()?;
                    let str = value.parse::<LitStr>()?;
                    path = Some(str.value());
                }
                Ok(())
            });
            return path;
        }
    }
    None
}
