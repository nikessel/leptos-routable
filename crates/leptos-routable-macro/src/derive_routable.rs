//! Contains the logic for `#[derive(Routable)]`.
//! Gathers each variant’s path/fallback, builds a `<Router>` + `<Routes>` tree.

use proc_macro::TokenStream;
use proc_macro2::Span as Span2;
use quote::quote;
use syn::{
    parse_macro_input,
    spanned::Spanned,
    Data::{Enum, Struct, Union},
    DeriveInput,
};
use deluxe::{ExtractAttributes, ParseMetaItem};

/// The routing mode (not fully used yet).
#[derive(Default, Debug, ParseMetaItem)]
enum RoutingMode {
    #[default]
    Auto,
    Flat,
    Nested,
}

#[derive(ExtractAttributes, Default, Debug)]
#[deluxe(attributes(routing))]
#[allow(unused)]
struct RoutingAttrs {
    #[deluxe(default)]
    pub mode: RoutingMode,
    #[deluxe(default)]
    pub transition: bool,
}

/// We parse `#[route(path = "...", meta="...")]` from each variant, if present.
#[derive(ExtractAttributes, Default, Debug)]
#[deluxe(attributes(route))]
struct RouteAttrs {
    #[deluxe(default)]
    pub path: Option<String>,
    #[deluxe(default)]
    #[allow(unused)]
    pub meta: Option<String>,
}

/// A small helper to store whether a variant was marked with `#[fallback]`.
#[derive(ExtractAttributes, Debug)]
#[deluxe(attributes(fallback))]
struct FallbackAttrs {
    pub redirect: Option<bool>,
    pub options: Option<String>,
}

/// Helper for building hooking function names: `"__ROUTE_COMP_{Enum}_{Variant}"`
fn build_registry_func_name(fn_name: &str) -> syn::Ident {
    let prefix = "__ROUTE_COMP_";
    let name = format!("{}{}", prefix, fn_name);
    syn::Ident::new(&name, fn_name.span())
}

/// Implementation of `#[derive(Routable)]`.
pub fn derive_routable_impl(input: TokenStream) -> TokenStream {
    let input_ast = parse_macro_input!(input as DeriveInput);
    let enum_ident = &input_ast.ident;

    // Ensure the macro is used on an enum
    let enum_data = match input_ast.data {
        Enum(ref e) => e,
        Struct(_) | Union(_) => {
            return syn::Error::new(
                input_ast.span(),
                "`#[derive(Routable)]` can only be used on enums.",
            )
                .to_compile_error()
                .into();
        }
    };

    // (`#[routing(mode = ...)]` here eventually)
    // let routing_attrs = match RoutingAttrs::extract_attributes(&mut input_ast.attrs) {
    //     Ok(a) => a,
    //     Err(e) => return e.to_compile_error().into(),
    // };

    let mut route_children = Vec::new();
    let mut fallback_func = None::<proc_macro2::TokenStream>;

    // For each variant => parse `#[route(...)]` + check if `#[fallback]`
    for mut variant in enum_data.variants.clone() {
        let var_ident = &variant.ident;
        let enum_variant_str = format!("{}_{}", enum_ident, var_ident);
        let registry_func_name = build_registry_func_name(&enum_variant_str);

        // Parse `#[route(path = ...)]`
        let route_attrs = match parse_route_attrs(&mut variant) {
            Ok(attrs) => attrs,
            Err(err) => return err.to_compile_error().into()
        };

        // Parse `#[fallback(...)]`
        let fallback_attrs = match parse_fallback_attrs(&mut variant, input_ast.span(), &fallback_func) {
            Ok(attrs) => attrs,
            Err(err) => return err.to_compile_error().into()
        };

        let path_str = route_attrs.path.unwrap_or_default();
        // let meta_str = route_attrs.meta; // example, not used yet

        match build_fallback_tokens(
            fallback_attrs.as_ref(),
            &path_str,
            &registry_func_name,
            variant.span().clone(),
        ) {
            Ok(fallback) => { fallback_func = fallback }
            Err(err) => { fallback_func = err.to_compile_error().into() }
        };

        // If the user gave `#[route(path = "...")]`, create a <Route> child
        if !path_str.is_empty() {
            let child = quote! {
                ::leptos_router::components::Route(
                    ::leptos_router::components::RouteProps::builder()
                        .path(::leptos_router::path!(#path_str))
                        .view(#registry_func_name)
                        .build()
                ),
            };
            route_children.push(child);
        }
    }

    // Build <Router> => <Routes> => route children
    let fallback_tokens = if let Some(fallback_func) = fallback_func {
        quote! { .fallback(#fallback_func) }
    } else {
        return syn::Error::new(
            input_ast.span(),
            "`#[fallback]` must be defined on one of the Routable enum variants.",
        )
            .to_compile_error()
            .into();
    };

    // TODO: use `routing_attrs.mode` to switch between `Routes` or `FlatRoutes`
    let route_component = quote! { ::leptos_router::components::Routes };
    let route_props = quote! { ::leptos_router::components::RoutesProps };

    let router_component = quote! {
        #[::leptos::component]
        pub fn #enum_ident() -> impl ::leptos::IntoView {
            ::leptos_router::components::Router(
                ::leptos_router::components::RouterProps::builder()
                    .children(
                        ::leptos::children::ToChildren::to_children(move ||
                            #route_component(
                                #route_props::builder()
                                    #fallback_tokens
                                    .children(::leptos::children::ToChildren::to_children(move ||
                                        (
                                            #(#route_children)*
                                        )
                                    ))
                                    .build()
                            )
                        )
                    )
                    .build()
            )
        }
    };

    let expanded = quote! {
        #router_component
    };

    expanded.into()
}

/// Parse `#[route(...)]` attributes on a single variant.
fn parse_route_attrs(variant: &mut syn::Variant) -> syn::Result<RouteAttrs> {
    RouteAttrs::extract_attributes(&mut variant.attrs)
        .map_err(|deluxe_err| {
            syn::Error::new(
                variant.span(),
                format!("Error parsing `#[route(...)]`: {}", deluxe_err),
            )
        })
}

/// Parse `#[fallback(...)]` attributes on a single variant.
fn parse_fallback_attrs(
    variant: &mut syn::Variant,
    input_span: Span2,
    fallback_func: &Option<proc_macro2::TokenStream>,
) -> syn::Result<Option<FallbackAttrs>> {
    match FallbackAttrs::extract_attributes(&mut variant.attrs) {
        Ok(attrs) => {
            validate_single_fallback(input_span, fallback_func)?;
            Ok(Some(attrs))
        }
        Err(err) => {
            // TODO: Decide how to handle “no fallback attribute found” vs. malformed attribute.
            Err(syn::Error::new(
                variant.span(),
                format!("Error parsing `#[fallback]`: {}", err),
            ))
        }
    }
}

fn validate_single_fallback(
    _input_span: Span2,
    _fallback_func: &Option<proc_macro2::TokenStream>,
) -> syn::Result<()> {
    // if fallback_func.is_some() {
    //     return Err(syn::Error::new(
    //         input_span,
    //         "`#[fallback]` may only be set on one variant of a `Routable` enum",
    //     ));
    // }
    Ok(())
}

/// Builds the fallback function (token stream) if this variant has fallback attributes.
/// Returns `None` if there is no fallback or if fallback attributes are absent.
///
/// If `fallback.redirect` is `true`, we parse `fallback.options` as a Rust expression
/// for `.options(...)`.
fn build_fallback_tokens(
    fallback_attrs: Option<&FallbackAttrs>,
    path_str: &str,
    registry_func_name: &proc_macro2::Ident,
    variant_span: proc_macro2::Span, // so we can produce good errors
) -> syn::Result<Option<proc_macro2::TokenStream>> {
    if let Some(fallback) = fallback_attrs {
        if let Some(true) = fallback.redirect {
            // If `redirect` is true, produce redirect logic
            let options_str = fallback
                .options
                .as_deref()
                .unwrap_or("Default::default()");

            // Parse `options_str` into a token stream for a valid Rust expression
            let options_ts: proc_macro2::TokenStream = options_str
                .parse()
                .map_err(|_| syn::Error::new(variant_span, "Invalid fallback options expression"))?;

            Ok(Some(quote! {
                || ::leptos_router::components::Redirect(
                    ::leptos_router::components::RedirectProps::builder()
                        .path(#path_str)
                        .options(#options_ts)
                        .build()
                )
            }))
        } else {
            // Otherwise, fall back to the variant's component
            Ok(Some(quote! {
                || #registry_func_name()
            }))
        }
    } else {
        Ok(None)
    }
}
