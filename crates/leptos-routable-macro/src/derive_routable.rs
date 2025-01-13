//! Contains the logic for `#[derive(Routable)]`.

use proc_macro::TokenStream;
use proc_macro2::{Span as Span2, TokenStream as TokenStream2};
use quote::{quote, quote_spanned};
use syn::{
    parse_macro_input, spanned::Spanned,
    Data::{Enum, Struct, Union},
    DeriveInput,
};
use darling::{FromAttributes, FromMeta};

#[derive(Default, Debug, FromMeta)]
enum RoutingMode {
    #[default]
    Auto,
    Flat,
    Nested,
}

#[derive(FromAttributes, Default, Debug)]
#[darling(attributes(routing))]
struct RoutesAttrs {
    #[darling(default)]
    pub mode: RoutingMode,
    #[darling(default)]
    pub transition: bool,
}

#[derive(Debug, FromAttributes)]
#[darling(attributes(route))]
pub struct RouteAttrs {
    pub path: Option<String>,
}

#[derive(Debug, FromAttributes)]
#[darling(attributes(parent_route))]
pub struct ParentRouteAttrs {
    pub path: Option<String>,
}

#[derive(Debug, FromAttributes)]
#[darling(attributes(protected_route))]
pub struct ProtectedRouteAttrs {
    pub path: Option<String>,
    pub condition: Option<String>,
    #[darling(default)]
    pub redirect_path: Option<String>,
    #[darling(default)]
    pub fallback: Option<String>,
}

#[derive(Debug, FromAttributes)]
#[darling(attributes(protected_parent_route))]
pub struct ProtectedParentRouteAttrs {
    pub path: Option<String>,
    pub condition: Option<String>,
    #[darling(default)]
    pub redirect_path: Option<String>,
    #[darling(default)]
    pub fallback: Option<String>,
}

#[derive(Debug, Default, FromAttributes)]
#[darling(attributes(fallback))]
pub struct FallbackAttrs {
    #[darling(default)]
    pub replace: Option<bool>,
    #[darling(default)]
    pub resolve: Option<bool>,
}

// Not all route kinds may be fully handled. Some logic is stubbed or not implemented yet.
enum RouteKind {
    Route(RouteAttrs),
    ParentRoute(ParentRouteAttrs),
    ProtectedRoute(ProtectedRouteAttrs),
    ProtectedParentRoute(ProtectedParentRouteAttrs),
    None,
}

impl Default for RouteKind {
    fn default() -> Self {
        RouteKind::None
    }
}

pub fn derive_routable_impl(input: TokenStream) -> TokenStream {
    let mut input_ast = parse_macro_input!(input as DeriveInput);

    // Only valid on enum
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

    let routing_attrs = match RoutesAttrs::from_attributes(&mut input_ast.attrs) {
        Ok(a) => a,
        Err(err) => {
            return syn::Error::new(
                input_ast.span(),
                format!("Error parsing `#[routes(...)]`: {}", err),
            )
                .into_compile_error()
                .into();
        }
    };

    let transition_tokens = if routing_attrs.transition {
        quote!(true)
    } else {
        quote!(false)
    };

    let (routes_component, routes_props) = match routing_attrs.mode {
        RoutingMode::Flat => (
            quote! { ::leptos_router::components::FlatRoutes },
            quote! { ::leptos_router::components::FlatRoutesProps },
        ),
        RoutingMode::Auto | RoutingMode::Nested => (
            quote! { ::leptos_router::components::Routes },
            quote! { ::leptos_router::components::RoutesProps },
        ),
    };

    let enum_ident = &input_ast.ident;
    let mut route_children = Vec::new();
    let mut fallback_func = None::<TokenStream2>;
    let mut extra_definitions = Vec::new();
    let mut hooking_fn_references = Vec::new();

    for variant in &enum_data.variants {
        let var_ident = &variant.ident;
        let enum_variant_str = format!("{}_{}", enum_ident, var_ident);
        let registry_func_name = crate::utils::build_registry_func_name(&enum_variant_str);

        // Determine the route kind
        let route_kind = match parse_route_kind(variant) {
            Ok(kind) => kind,
            Err(err) => return err.to_compile_error().into(),
        };

        // If there's a path, we could verify its dynamic parameters, etc.
        // parse_and_check_path_params(…) // not implemented fully here

        // Parse fallback (only one allowed). If found, store or wrap it.
        match parse_fallback_attrs(variant, input_ast.span(), &fallback_func) {
            Ok(Some(fb_attrs)) => {
                let path_str = match route_kind {
                    RouteKind::Route(ref attr) => &attr.path,
                    _ => panic!("")
                };
                let wrapped_name = syn::Ident::new(
                    &format!("FallbackWrapped_{}_{}", enum_ident, var_ident),
                    variant.span(),
                );
                // Building a fallback-wrapped component, or not
                let fallback_def = build_wrapped_fallback_component(
                    &wrapped_name,
                    &registry_func_name,
                    &path_str.clone().expect("Fallback expects path"), // TODO, fix logic
                    &fb_attrs,
                );
                extra_definitions.push(fallback_def);
                fallback_func = Some(quote! { || #wrapped_name() });
            }
            Ok(None) => {}
            Err(err) => return err.to_compile_error().into(),
        }

        // Build the child route if it's some recognized route kind
        if let Some(child_ts) = build_route_child(&route_kind, &registry_func_name) {
            route_children.push(child_ts);
        }

        // Force a reference to the hooking function
        hooking_fn_references.push(quote_spanned! { variant.span()=>
            const _: () = {
                let _check = #registry_func_name;
            };
        });
    }

    // If there's no fallback, that's an error in this logic
    let fallback_tokens = match fallback_func {
        Some(f) => f,
        None => {
            return syn::Error::new(
                input_ast.span(),
                "No variant is marked with `#[fallback]`. Exactly one is required.",
            )
                .to_compile_error()
                .into();
        }
    };

    // Build top-level Router component
    let router_component = quote! {
        #[::leptos::component]
        pub fn #enum_ident() -> impl ::leptos::IntoView {
            ::leptos_router::components::Router(
                ::leptos_router::components::RouterProps::builder()
                    .children(::leptos::children::ToChildren::to_children(move ||
                        #routes_component(
                            #routes_props::builder()
                                .transition(#transition_tokens)
                                .fallback(#fallback_tokens)
                                .children(::leptos::children::ToChildren::to_children(move ||
                                    (#(#route_children)*)
                                ))
                                .build()
                        )
                    ))
                    .build()
            )
        }
    };

    let expanded = quote! {
        #(#hooking_fn_references)*
        #(#extra_definitions)*
        #router_component
    };

    crate::utils::format_generated_code(expanded).into()
}

// not implemented fully:
fn parse_route_kind(variant: &syn::Variant) -> syn::Result<RouteKind> {
    // Minimal stub
    let mut found: Option<RouteKind> = None;

    if variant.attrs.iter().any(|attr| attr.path().is_ident("route")) {
        let route_attrs = RouteAttrs::from_attributes(&variant.attrs)?;
        if let Some(path) = &route_attrs.path {
            if !path.is_empty() {
                if found.is_some() {
                    return Err(multiple_route_error(variant));
                }
                found = Some(RouteKind::Route(route_attrs));
            }
        }
    }

    if variant
        .attrs
        .iter()
        .any(|attr| attr.path().is_ident("parent_route"))
    {
        let parent_attrs = ParentRouteAttrs::from_attributes(&variant.attrs)?;
        if let Some(path) = &parent_attrs.path {
            if !path.is_empty() {
                if found.is_some() {
                    return Err(multiple_route_error(variant));
                }
                found = Some(RouteKind::ParentRoute(parent_attrs));
            }
        }
    }

    if variant
        .attrs
        .iter()
        .any(|attr| attr.path().is_ident("protected_route"))
    {
        let prot_attrs = ProtectedRouteAttrs::from_attributes(&variant.attrs)?;
        if let Some(path) = &prot_attrs.path {
            if !path.is_empty() {
                if found.is_some() {
                    return Err(multiple_route_error(variant));
                }
                found = Some(RouteKind::ProtectedRoute(prot_attrs));
            }
        }
    }

    if variant
        .attrs
        .iter()
        .any(|attr| attr.path().is_ident("protected_parent_route"))
    {
        let prot_par_attrs = ProtectedParentRouteAttrs::from_attributes(&variant.attrs)?;
        if let Some(path) = &prot_par_attrs.path {
            if !path.is_empty() {
                if found.is_some() {
                    return Err(multiple_route_error(variant));
                }
                found = Some(RouteKind::ProtectedParentRoute(prot_par_attrs));
            }
        }
    }

    Ok(found.unwrap_or(RouteKind::None))
}

fn multiple_route_error(variant: &syn::Variant) -> syn::Error {
    syn::Error::new(
        variant.span(),
        "Multiple route-like attributes found on this variant. \
         Only one of `#[route]`, `#[parent_route]`, `#[protected_route]`, \
         or `#[protected_parent_route]` is allowed.",
    )
}

// not implemented fully:
fn build_route_child(kind: &RouteKind, registry_func_name: &syn::Ident) -> Option<TokenStream2> {
    match kind {
        RouteKind::None => None,
        RouteKind::Route(attrs) => {
            let path_str = attrs.path.as_deref().unwrap_or_default();
            Some(quote! {
                ::leptos_router::components::Route(
                    ::leptos_router::components::RouteProps::builder()
                        .path(::leptos_router::path!(#path_str))
                        .view(#registry_func_name)
                        .build()
                ),
            })
        }
        RouteKind::ParentRoute(attrs) => {
            let path_str = attrs.path.as_deref().unwrap_or_default();
            Some(quote! {
                ::leptos_router::components::ParentRoute(
                    ::leptos_router::components::ParentRouteProps::builder()
                        .path(::leptos_router::path!(#path_str))
                        .view(#registry_func_name)
                        .children(::leptos::children::ToChildren::to_children(move || {
                            // child route(s) could go here
                            ()
                        }))
                        .build()
                ),
            })
        }
        RouteKind::ProtectedRoute(attrs) => {
            let path_str = attrs.path.as_deref().unwrap_or("");
            let condition_str = attrs.condition.as_deref().unwrap_or("");
            let redirect_path_str = attrs.redirect_path.as_deref().unwrap_or("/");
            let fallback_str = attrs.fallback.as_deref().unwrap_or("|| ()");

            Some(quote! {
                ::leptos_router::components::ProtectedRoute(
                    ::leptos_router::components::ProtectedRouteProps::builder()
                        .path(::leptos_router::path!(#path_str))
                        .view(#registry_func_name)
                        .condition(::leptos::callback::Callback::new(move |_| {
                            #condition_str()
                        }))
                        .redirect_path(::leptos::callback::Callback::new(move |_| {
                            format!("{}", #redirect_path_str)
                        }))
                        .fallback(::leptos::children::ViewFn::new(#fallback_str))
                        .build()
                ),
            })
        }
        RouteKind::ProtectedParentRoute(attrs) => {
            let path_str = attrs.path.as_deref().unwrap_or("");
            let condition_str = attrs.condition.as_deref().unwrap_or("");
            let redirect_path_str = attrs.redirect_path.as_deref().unwrap_or("/");
            let fallback_str = attrs.fallback.as_deref().unwrap_or("|| ()");

            Some(quote! {
                ::leptos_router::components::ProtectedParentRoute(
                    ::leptos_router::components::ProtectedParentRouteProps::builder()
                        .path(::leptos_router::path!(#path_str))
                        .view(#registry_func_name)
                        .condition(::leptos::callback::Callback::new(move |_| {
                            #condition_str()
                        }))
                        .redirect_path(::leptos::callback::Callback::new(move |_| {
                            format!("{}", #redirect_path_str)
                        }))
                        .fallback(::leptos::children::ViewFn::new(#fallback_str))
                        .children(::leptos::children::ToChildren::to_children(move || {
                            // child route(s) could go here
                            ()
                        }))
                        .build()
                ),
            })
        }
    }
}

// not implemented fully:
fn parse_fallback_attrs(
    variant: &syn::Variant,
    input_span: Span2,
    fallback_func: &Option<TokenStream2>,
) -> syn::Result<Option<FallbackAttrs>> {
    let fallback_attrs: Vec<_> = variant
        .attrs
        .iter()
        .filter(|attr| attr.path().is_ident("fallback"))
        .collect();

    if fallback_attrs.is_empty() {
        return Ok(None);
    }
    if fallback_attrs.len() > 1 {
        return Err(syn::Error::new(
            variant.span(),
            "Multiple `#[fallback(...)]` attributes found on the same variant.",
        ));
    }

    validate_single_fallback(input_span, fallback_func)?;

    match FallbackAttrs::from_attributes(&variant.attrs) {
        Ok(attrs) => Ok(Some(attrs)),
        Err(err) => Err(syn::Error::new(
            variant.span(),
            format!("Error parsing `#[fallback]`: {}", err),
        )),
    }
}

// not implemented fully:
fn validate_single_fallback(
    input_span: Span2,
    fallback_func: &Option<TokenStream2>,
) -> syn::Result<()> {
    if fallback_func.is_some() {
        return Err(syn::Error::new(
            input_span,
            "`#[fallback]` may only be set on one variant of a `Routable` enum.",
        ));
    }
    Ok(())
}

// not implemented fully:
fn build_wrapped_fallback_component(
    wrapped_ident: &syn::Ident,
    user_variant_ident: &syn::Ident,
    path_str: &str,
    fb: &FallbackAttrs,
) -> TokenStream2 {
    let replace = fb.replace.unwrap_or(false);
    let resolve = fb.resolve.unwrap_or(false);

    quote! {
        #[::leptos::component]
        pub fn #wrapped_ident() -> impl ::leptos::IntoView {
            let user_view = #user_variant_ident();
            ::leptos::prelude::Effect::new(move || {
                // not implemented: fallback behavior (navigate, etc.)
                let navigate = ::leptos_router::hooks::use_navigate();
                navigate(
                    #path_str,
                    ::leptos_router::NavigateOptions {
                        resolve: #resolve,
                        replace: #replace,
                        scroll: true,
                        state: ::leptos_router::location::State::new(None),
                    }
                );
            });
            user_view
        }
    }
}
