use proc_macro::TokenStream;
use std::collections::HashSet;
use darling::ast::NestedMeta;
use quote::quote;
use syn::{parse_macro_input, spanned::Spanned, Data, DataEnum, DeriveInput, Fields, Meta, Lit, Attribute, Variant, Ident, Type, Error, LitStr};


/// Derive macro that implements `ToPath` for an enum, looking for:
/// ```ignore
/// #[route(path = "/foo/:id")]
/// ```
/// on each variant. If the variant is, e.g.
/// ```ignore
///   AssetDetails { id: u64, filter: Option<String> },
/// ```
/// then the path might expand to `"/foo/123?filter=xyz"` at runtime.
pub fn derive_to_path_impl(input: TokenStream) -> TokenStream {
    // 1) Parse the input (the enum) via `syn`.
    let ast = parse_macro_input!(input as DeriveInput);

    // Ensure it's an enum.
    let Data::Enum(data_enum) = ast.data else {
        return syn::Error::new_spanned(
            ast,
            "`#[derive(ToPath)]` can only be used on an enum.",
        )
            .to_compile_error()
            .into();
    };

    let enum_ident = &ast.ident;

    // We'll accumulate match arms for `impl ToPath`.
    let mut match_arms = Vec::new();
    let mut errors = Vec::new();

    for variant in data_enum.variants {
        // Attempt to process each variant. If there's an error, collect it.
        match process_variant(enum_ident, variant) {
            Ok(arm_ts) => {
                if let Some(ts) = arm_ts {
                    match_arms.push(ts);
                }
            }
            Err(e) => errors.push(e),
        }
    }

    // If we accumulated errors, return them combined.
    if !errors.is_empty() {
        let mut combined = proc_macro2::TokenStream::new();
        for e in errors {
            combined.extend(e.to_compile_error());
        }
        return combined.into();
    }

    // If no arms, we’ll just default to empty path in the final match.
    let fallback_arm = quote! {
        _ => {
            // No recognized route => just "/"
            "/".to_string()
        }
    };

    // 2) Build the final `impl ToPath for MyEnum`
    let expanded = quote! {
        impl ::leptos_routable::prelude::ToPath for #enum_ident {
            fn to_path(&self) -> String {
                match self {
                    #( #match_arms, )*
                    #fallback_arm
                }
            }
        }
    };
    eprintln!("{}", expanded);
    expanded.into()
}

/* -------------------------------------------------------------------------------------------------
 * MAIN VARIANT PROCESSOR
 * -----------------------------------------------------------------------------------------------*/
/// Processes a single variant to produce its match arm (or `Ok(None)` if no `#[route]`).
fn process_variant(
    enum_ident: &Ident,
    variant: Variant,
) -> syn::Result<Option<proc_macro2::TokenStream>> {
    let Variant { ident, fields, attrs, .. } = variant;

    // 1) Extract `#[route(path="...")]` if present. If absent or empty, skip this variant.
    let route_path = match find_route_path(&attrs) {
        Some(p) if !p.is_empty() => p,
        _ => return Ok(None), // no route => no to_path match arm
    };

    // 2) Extract named fields (with spans).
    let field_infos = extract_variant_fields(enum_ident, &ident, &fields)?;

    // 3) Validate the path string vs. the fields (e.g. `:foo?` → must be Option<T>, leftover → Option).
    validate_path_and_fields(&route_path, &field_infos, &ident)?;

    // 4) Build the destructuring pattern + generate final code.
    let (variant_pat, fields_for_build) = build_variant_pattern(enum_ident, &ident, &fields)?;

    // 5) Create the body code that constructs the final path at runtime.
    let build_code = generate_path_builder(&route_path, &fields_for_build);

    let arm = quote! {
        #variant_pat => {
            #build_code
        }
    };
    Ok(Some(arm))
}

/* -------------------------------------------------------------------------------------------------
 * #1: Extract Variant Fields
 * -----------------------------------------------------------------------------------------------*/
/// Extracts named or unnamed fields from the variant, ensuring only one unnamed if present.
/// Returns a vector of `(String, Type, Span)` for each field to enable validations and error spans.
struct FieldMeta {
    name: String,
    ty: Type,
    span: proc_macro2::Span,
}

/// Gathers the "meta" (name, type, span) of each field in the variant.
fn extract_variant_fields(
    enum_ident: &Ident,
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
            // For nested routing, we ONLY allow exactly 1 unnamed field:
            let count = unnamed.unnamed.len();
            if count == 0 {
                let msg = format!(
                    "Variant `{}` has 0 unnamed fields, but we expect exactly 1 for nested routing.",
                    variant_ident
                );
                return Err(Error::new(unnamed.span(), msg));
            }
            if count > 1 {
                let msg = format!(
                    "Variant `{}` has {} unnamed fields, but only exactly 1 is allowed for nested routing.",
                    variant_ident, count
                );
                return Err(Error::new(unnamed.span(), msg));
            }

            let only_field = &unnamed.unnamed[0];
            let spn = only_field.span();

            // We'll store the name as `_0`
            Ok(vec![FieldMeta {
                name: "_0".to_string(),
                ty: only_field.ty.clone(),
                span: spn,
            }])
        }
    }
}

/* -------------------------------------------------------------------------------------------------
 * #2: Validate Path + Fields
 * -----------------------------------------------------------------------------------------------*/
/// Represents a route segment (e.g., `:foo`, `:foo?`, or static).
#[derive(Debug)]
enum RouteSegment {
    Static(String),
    Param(String),
    OptionalParam(String),
}

/// Ensures each param in the route is present in fields, optional params are `Option`, etc.
fn validate_path_and_fields(
    route_str: &str,
    fields: &[FieldMeta],
    variant_ident: &Ident,
) -> syn::Result<()> {
    let segments = parse_segments(route_str);
    let mut used_fields = Vec::new();

    // 1) For each segment `:foo` or `:foo?`, ensure there's a matching field.
    for seg in &segments {
        match seg {
            RouteSegment::Static(_) => {}
            RouteSegment::Param(name) => {
                used_fields.push(name.clone());
                // Must exist among fields
                match fields.iter().find(|f| f.name == *name) {
                    Some(_) => {} // all good
                    None => {
                        let msg = format!(
                            "Path parameter `:{}` not found as a named field in variant `{}`.",
                            name, variant_ident
                        );
                        return Err(Error::new(variant_ident.span(), msg));
                    }
                }
            }
            RouteSegment::OptionalParam(name) => {
                used_fields.push(name.clone());
                // Must exist + must be Option
                match fields.iter().find(|f| f.name == *name) {
                    Some(field_meta) => {
                        if !is_option_type(&field_meta.ty) {
                            let msg = format!(
                                "Optional path parameter `:{}?` in variant `{}` must be `Option<T>`, found `{:?}`.",
                                name, variant_ident, field_meta.ty
                            );
                            return Err(Error::new(field_meta.span, msg));
                        }
                    }
                    None => {
                        let msg = format!(
                            "Optional path parameter `:{}?` not found as a named field in variant `{}`.",
                            name, variant_ident
                        );
                        return Err(Error::new(variant_ident.span(), msg));
                    }
                }
            }
        }
    }

    // 2) For leftover fields not used in the path, they must be `Option` if we want them as query params.
    for f in fields {
        if !used_fields.contains(&f.name) {
            // leftover => must be Option if we want to treat as query
            if !is_option_type(&f.ty) {
                let msg = format!(
                    "Field `{}` in variant `{}` is not in the path, so it must be `Option<T>` if used as a query param.",
                    f.name, variant_ident
                );
                return Err(Error::new(f.span, msg));
            }
        }
    }

    Ok(())
}

/* -------------------------------------------------------------------------------------------------
 * Build Variant Pattern (for final codegen)
 * -----------------------------------------------------------------------------------------------*/
/// Create a destructuring pattern + gather (field_name, field_type).
///
/// For `VariantName { id, filter }`, we produce:
/// ```ignore
/// Self::VariantName { id, filter } => { ... }
/// ```
fn build_variant_pattern(
    enum_ident: &Ident,
    variant_ident: &Ident,
    fields: &Fields,
) -> syn::Result<(proc_macro2::TokenStream, Vec<(String, Type)>)> {
    match fields {
        Fields::Unit => {
            // e.g. `Enum::Variant`
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
            // We already enforced exactly 1 field in `extract_variant_fields`.
            // So let's just build a pattern `Enum::Variant(_0)`
            let f = &unnamed.unnamed[0];
            let spn = f.span();
            let field_ident = syn::Ident::new("_0", spn);

            let pat = quote!( #enum_ident::#variant_ident(#field_ident) );
            let fields_for_build = vec![("_0".to_string(), f.ty.clone())];
            Ok((pat, fields_for_build))
        }
    }
}

/* -------------------------------------------------------------------------------------------------
 * Generate Path Builder
 * -----------------------------------------------------------------------------------------------*/
/// Generate code that builds the path string from the route pattern + leftover fields => query.
fn generate_path_builder(
    route: &str,
    fields: &[(String, Type)],
) -> proc_macro2::TokenStream {
    let segments = parse_segments(route);
    let mut used_fields = Vec::new();

    let segment_stmts: Vec<_> = segments.into_iter().map(|seg| match seg {
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
        },
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
    }).collect();

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

        // path segments
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

/* -------------------------------------------------------------------------------------------------
 * Route Path Segment Parsing
 * -----------------------------------------------------------------------------------------------*/
fn parse_segments(route: &str) -> Vec<RouteSegment> {
    let without_leading = route.trim_start_matches('/');
    let mut segs = Vec::new();

    for part in without_leading.split('/') {
        if part.starts_with(':') {
            // e.g. ":subpath?" or ":id"
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

/// Checks if a type is `Option<...>`.
fn is_option_type(ty: &Type) -> bool {
    if let syn::Type::Path(tp) = ty {
        if let Some(seg) = tp.path.segments.last() {
            return seg.ident == "Option";
        }
    }
    false
}

/* -------------------------------------------------------------------------------------------------
 * find_route_path
 * -----------------------------------------------------------------------------------------------*/
/// Read `#[route(path="...")]` from the variant attributes.
fn find_route_path(attrs: &[Attribute]) -> Option<String> {
    for attr in attrs {
        if attr.path().is_ident("route") {
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
