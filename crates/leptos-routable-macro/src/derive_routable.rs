//! Contains the logic for `#[derive(Routable)]`.
//! Gathers each variant’s path/fallback, builds a `<Router>` + `<Routes>` tree.

use proc_macro::TokenStream;
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
#[allow(unused)]
#[derive(Default, Debug)]
struct FallbackAttr {
    pub fallback: bool,
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
        Enum(e) => e,
        Struct(_) | Union(_) => {
            return syn::Error::new(
                input_ast.span(),
                "`#[derive(Routable)]` can only be used on enums.",
            )
                .to_compile_error()
                .into();
        }
    };

    // (`#[routing(mode = ...)]` here if desired)
    // let routing_attrs = match RoutingAttrs::extract_attributes(&mut input_ast.attrs) {
    //     Ok(a) => a,
    //     Err(e) => return e.to_compile_error().into(),
    // };

    let mut route_children = Vec::new();
    let mut fallback_func = None::<proc_macro2::TokenStream>;

    // For each variant => parse `#[route(...)]` + check if `#[fallback]`
    for mut variant in enum_data.variants {
        let var_ident = &variant.ident;
        let enum_variant_str = format!("{}_{}", enum_ident, var_ident);
        let registry_func_name = build_registry_func_name(&enum_variant_str);

        // Parse `#[route(path = ...)]`
        let route_attrs = match RouteAttrs::extract_attributes(&mut variant.attrs) {
            Ok(a) => a,
            Err(e) => return e.to_compile_error().into(),
        };
        let path_str = route_attrs.path.unwrap_or_default();
        // let meta_str = route_attrs.meta; // example, not used yet

        // Check for `#[fallback]`
        let is_fallback = variant
            .attrs
            .iter()
            .any(|attr| attr.path().is_ident("fallback"));

        if is_fallback {
            // Store hooking function for fallback usage
            fallback_func = Some(quote! {
                || #registry_func_name()
            }.into());
        }

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
    let fallback_tokens = if let Some(func) = fallback_func {
        quote! { .fallback(#func) }
    } else {
        quote! { .fallback(|| ()) }
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
