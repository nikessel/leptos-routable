use proc_macro::TokenStream;
use proc_macro2::{Span as Span2, TokenStream as TokenStream2};
use quote::quote;
use syn::{parse_macro_input, spanned::Spanned, Data::{Enum, Struct, Union}, DeriveInput, Ident, Type, Variant, Fields};
use darling::{FromDeriveInput, FromVariant};

/* -------------------------------------------------------------------------------------------------
 * Helper Functions
 * -----------------------------------------------------------------------------------------------*/
fn to_snake_case(s: &str) -> String {
    let mut result = String::new();
    let mut prev_upper = false;

    for (i, c) in s.chars().enumerate() {
        if c.is_uppercase() {
            if i > 0 && !prev_upper {
                result.push('_');
            }
            result.push(c.to_lowercase().next().unwrap());
            prev_upper = true;
        } else {
            result.push(c);
            prev_upper = false;
        }
    }

    result
}

trait IntoChildTokens {
    fn into_child_tokens(self, view: Ident) -> Option<TokenStream2>;
}

/* -------------------------------------------------------------------------------------------------
 * leptos_router::components::Route
 * -----------------------------------------------------------------------------------------------*/
#[derive(std::fmt::Debug, FromVariant)]
#[darling(attributes(route))]
struct RouteVariant {
    #[allow(unused)] ident: Ident,
    #[allow(unused)] fields: darling::ast::Fields<syn::Type>,

    // Arguments
    path: syn::LitStr,
}

impl IntoChildTokens for RouteVariant {
    fn into_child_tokens(self, view: Ident) -> Option<TokenStream2> {
        let path = self.path;
        Some(quote! {
            ::leptos_router::components::Route(
                ::leptos_router::components::RouteProps::builder()
                    .path(::leptos_router::path!(#path))
                    .view(#view)
                    .build())
        })
    }
}

/* -------------------------------------------------------------------------------------------------
 * leptos_router::components::ParentRoute
 * -----------------------------------------------------------------------------------------------*/
#[derive(std::fmt::Debug, FromVariant)]
#[darling(attributes(parent_route))]
struct ParentRouteVariant {
    #[allow(unused)] ident: Ident,
    fields: darling::ast::Fields<syn::Type>,
    #[allow(unused)] routable: Option<Ident>,

    // Arguments
    path: syn::LitStr,
    ssr: Option<syn::Expr>,
}

impl IntoChildTokens for ParentRouteVariant {
    fn into_child_tokens(self, view: Ident) -> Option<TokenStream2> {
        let path = self.path;
        let ssr = self.ssr.unwrap_or(syn::parse_quote!(Default::default()));
        // There can only be one, error elsewhere ensures.
        let inner_ident = self.fields.fields.into_iter().next()?;
        Some(quote! { #inner_ident::parent_route(::leptos_router::path!(#path), #view, #ssr) })
    }
}

/* -------------------------------------------------------------------------------------------------
 * leptos_router::components::ProtectedRoute
 * -----------------------------------------------------------------------------------------------*/
#[derive(std::fmt::Debug, FromVariant)]
#[darling(attributes(protected_route))]
struct ProtectedRouteVariant {
    #[allow(unused)] ident: Ident,
    #[allow(unused)] fields: darling::ast::Fields<syn::Type>,

    // Arguments
    path: syn::LitStr,
    condition: syn::Expr,
    redirect_path: syn::Expr,
    fallback: syn::Expr,
}

impl IntoChildTokens for ProtectedRouteVariant {
    fn into_child_tokens(self, view: Ident) -> Option<TokenStream2> {
        let path = self.path;
        let condition = self.condition;
        let redirect_path = self.redirect_path;
        let fallback = self.fallback;
        Some(quote! {
             ::leptos_router::components::ProtectedRoute(
                 ::leptos_router::components::ProtectedRouteProps::builder()
                     .path(::leptos_router::path!(#path))
                     .view(#view)
                     .condition(#condition)
                     .redirect_path(#redirect_path)
                     .fallback(#fallback)
                     .build()
             )
        })
    }
}

/* -------------------------------------------------------------------------------------------------
 * leptos_router::components::ProtectedParentRoute
 * -----------------------------------------------------------------------------------------------*/
#[derive(std::fmt::Debug, FromVariant)]
#[darling(attributes(protected_parent_route))]
struct ProtectedParentRouteVariant {
    #[allow(unused)] ident: Ident,
    fields: darling::ast::Fields<syn::Type>,

    // Arguments
    path: syn::LitStr,
    condition: syn::Expr,
    redirect_path: syn::Expr,
    fallback: syn::Expr,
    ssr: Option<syn::Expr>,
}

impl IntoChildTokens for ProtectedParentRouteVariant {
    fn into_child_tokens(self, view: Ident) -> Option<TokenStream2> {
        let path = self.path;
        let condition = self.condition;
        let redirect_path = self.redirect_path;
        let fallback = self.fallback;
        let ssr = self.ssr.unwrap_or(syn::parse_quote!(Default::default()));
        // There can only be one, error elsewhere ensures.
        let inner_ident = self.fields.fields.into_iter().next()?;
        Some(quote! { #inner_ident::protected_parent_route(::leptos_router::path!(#path), #view, #condition, #fallback.into(), #redirect_path, #ssr) })
    }
}

/* -------------------------------------------------------------------------------------------------
 * Fallback
 * -----------------------------------------------------------------------------------------------*/
#[derive(std::fmt::Debug, FromVariant)]
#[darling(attributes(fallback))]
#[allow(unused)]
pub struct StandaloneFallbackVariant {
    ident: Ident,
    discriminant: Option<syn::Expr>,
    fields: darling::ast::Fields<syn::Type>,
}

/* -------------------------------------------------------------------------------------------------
 * `#[derive(Routable)] -> #[routable(...)]`
 * -----------------------------------------------------------------------------------------------*/
#[derive(FromDeriveInput, std::fmt::Debug)]
#[darling(attributes(routes), supports(enum_any))]
pub(crate) struct RoutableConfiguration {
    ident: syn::Ident,
    //#[allow(unused)]
    //attrs: Vec<syn::Attribute>,

    #[darling(default)]
    pub(crate) transition: bool,

    #[darling(default)]
    pub(crate) view_prefix: String,

    #[darling(default = "default_view_suffix")]
    pub(crate) view_suffix: String,

    #[darling(default)]
    pub(crate) state_suffix: Option<String>,
}

impl IntoChildTokens for RouteKind {
    fn into_child_tokens(self, view: Ident) -> Option<TokenStream2> {
        match self {
            Self::Route(route) => route.into_child_tokens(view),
            Self::ParentRoute(parent) => parent.into_child_tokens(view),
            Self::ProtectedRoute(protected) => protected.into_child_tokens(view),
            Self::ProtectedParentRoute(protected_parent) => protected_parent.into_child_tokens(view),
            Self::None => None
        }
    }
}

fn default_view_suffix() -> String {
    "View".to_string()
}

trait FromVariantWithKind: Sized {
    fn attr_ident() -> &'static str;
    fn into_kind(self) -> RouteKind;
}

// Implement for each variant type
impl FromVariantWithKind for RouteVariant {
    fn attr_ident() -> &'static str { "route" }
    fn into_kind(self) -> RouteKind { RouteKind::Route(self) }
}

impl FromVariantWithKind for ParentRouteVariant {
    fn attr_ident() -> &'static str { "parent_route" }
    fn into_kind(self) -> RouteKind { RouteKind::ParentRoute(self) }
}

impl FromVariantWithKind for ProtectedRouteVariant {
    fn attr_ident() -> &'static str { "protected_route" }
    fn into_kind(self) -> RouteKind { RouteKind::ProtectedRoute(self) }
}

impl FromVariantWithKind for ProtectedParentRouteVariant {
    fn attr_ident() -> &'static str { "protected_parent_route" }
    fn into_kind(self) -> RouteKind { RouteKind::ProtectedParentRoute(self) }
}


macro_rules! try_parse_variants {
    ($variant:expr, $($T:ty),+) => {{
        let mut found = None;
        $(
            if $variant.attrs.iter().any(|attr| attr.path().is_ident(<$T>::attr_ident())) {
                match <$T>::from_variant($variant) {
                    Ok(parsed) => {
                        if found.is_some() {
                            return Err(multiple_route_error($variant));
                        }
                        found = Some(parsed.into_kind());
                    }
                    Err(err) => return Err(err),
                }
            }
        )+
        found
    }};
}


/* -------------------------------------------------------------------------------------------------
 * RouteKind
 * -----------------------------------------------------------------------------------------------*/

#[derive(std::fmt::Debug)]
#[allow(unused)]
enum RouteKind {
    Route(RouteVariant),
    ParentRoute(ParentRouteVariant),
    ProtectedRoute(ProtectedRouteVariant),
    ProtectedParentRoute(ProtectedParentRouteVariant),
    None,
}

/* -------------------------------------------------------------------------------------------------
 * `#[derive(Routable)]` implementation
 * -----------------------------------------------------------------------------------------------*/
pub fn derive_routable_impl(input: TokenStream) -> TokenStream {
    let input_ast = parse_macro_input!(input as DeriveInput);
    let config = match RoutableConfiguration::from_derive_input(&input_ast) {
        Ok(config) => config,
        Err(err) => return err.write_errors().into(),
    };
    let data = match input_ast.data {
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

    let mut children = Vec::new();
    let mut fallback = None::<TokenStream2>;

    // Determine if we need state support
    let state_store_type = config.state_suffix.as_ref().map(|suffix| {
        let state_name = format!("{}{}", config.ident, suffix);
        syn::Ident::new(&state_name, config.ident.span())
    });


    for variant in &data.variants {
        let view_ident = crate::utils::build_variant_view_name(&config.ident, &variant.ident, &config);
        let route_kind = match parse_variant(variant) {
            Ok(kind) => kind,
            Err(err) => return err.write_errors().into(),
        };

        match parse_fallback_attrs(variant, input_ast.span(), &fallback) {
            Ok(()) => { fallback = Some(quote! { #view_ident }); }
            Err(err) => return err.to_compile_error().into(),
        }

        // No longer generate per-route wrappers
        let view_to_use = view_ident;

        if let Some(kind) = route_kind {
            if let Some(child_ts) = kind.into_child_tokens(view_to_use) {
                children.push(child_ts);
            }
        }
    }

    let fallback = match fallback {
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
    let enum_ident = config.ident;
    let transition = config.transition;

    // Generate compile-time validation if state_suffix is provided
    let field_validation = if let Some(ref state_store_type) = state_store_type {
        let mut all_checks = Vec::new();

        // Check that ALL routes have corresponding state structs (fields in root state)
        for variant in &data.variants {
            let field_name = syn::Ident::new(
                &to_snake_case(&variant.ident.to_string()),
                variant.ident.span()
            );

            // Every route must have a field in the root state
            all_checks.push(quote! {
                // Every route must have a state field
                let _ = |store: &#state_store_type| {
                    let _ = store.#field_name;
                };
            });

            // If this is a parent route, it must have a sub_state field
            if matches!(&variant.fields, syn::Fields::Unnamed(_)) {
                // Generate the expected SubState type name
                let sub_state_type = syn::Ident::new(
                    &format!("{}SubState", variant.ident),
                    variant.ident.span()
                );

                // Check the type of the state field's sub_state
                let field_state_type = syn::Ident::new(
                    &format!("{}State", variant.ident),
                    variant.ident.span()
                );

                all_checks.push(quote! {
                    // Parent routes must have a sub_state field of the correct type
                    let _ = |state: &#field_state_type| {
                        let _: &#sub_state_type = &state.sub_state;
                    };
                });

                // Also check that the SubState has fields for all nested routes
                if let syn::Fields::Unnamed(fields) = &variant.fields {
                    if let Some(syn::Field { ty: syn::Type::Path(type_path), .. }) = fields.unnamed.first() {
                        if let Some(_nested_enum) = type_path.path.segments.last() {
                            // This would need to parse the nested enum to get its variants
                            // For now we'll trust the user to define the SubState correctly
                        }
                    }
                }
            }
        }

        quote! {
            // Compile-time validation that all routes have state
            const _: () = {
                #(#all_checks)*
            };
        }
    } else {
        quote! {}
    };

    // Generate context helper impls for all enums
    let context_helpers = {
        let mut helper_impls = Vec::new();

        // Only root enum gets Store<T> helper
        if let Some(root_state) = state_store_type.as_ref() {
            helper_impls.push(quote! {
                impl #root_state {
                    pub fn use_context() -> Option<reactive_stores::Store<#root_state>> {
                        leptos::prelude::use_context::<reactive_stores::Store<#root_state>>()
                    }

                    pub fn expect_context() -> reactive_stores::Store<#root_state> {
                        Self::use_context()
                            .expect(concat!(stringify!(#root_state), " should be provided by router"))
                    }
                }
            });
        }

        // Generate impl for each route's state type (uses Field<T>)
        for variant in &data.variants {
            let state_type = syn::Ident::new(
                &format!("{}State", variant.ident),
                variant.ident.span()
            );

            helper_impls.push(quote! {
                impl #state_type {
                    pub fn use_context() -> Option<reactive_stores::Field<#state_type>> {
                        leptos::prelude::use_context::<reactive_stores::Field<#state_type>>()
                    }

                    pub fn expect_context() -> reactive_stores::Field<#state_type> {
                        Self::use_context()
                            .expect(concat!(stringify!(#state_type), " should be provided by router"))
                    }
                }
            });

            // If it's a parent route, also generate helpers for SubState
            if matches!(&variant.fields, syn::Fields::Unnamed(_)) {
                let sub_state_type = syn::Ident::new(
                    &format!("{}SubState", variant.ident),
                    variant.ident.span()
                );

                helper_impls.push(quote! {
                    impl #sub_state_type {
                        pub fn use_context() -> Option<reactive_stores::Field<#sub_state_type>> {
                            leptos::prelude::use_context::<reactive_stores::Field<#sub_state_type>>()
                        }

                        pub fn expect_context() -> reactive_stores::Field<#sub_state_type> {
                            Self::use_context()
                                .expect(concat!(stringify!(#sub_state_type), " should be provided by parent route"))
                        }
                    }
                });
            }
        }

        quote! {
            #(#helper_impls)*
        }
    };

    // Generate state initialization for routes() method (only for root enum)
    let state_init = if let Some(ref state_store_type) = state_store_type {
        let provide_statements = generate_recursive_provides(data, quote! { __root_store });

        quote! {
            let __root_store = reactive_stores::Store::new(<#state_store_type as Default>::default());
            leptos::prelude::provide_context(__root_store.clone());
            #(#provide_statements)*
        }
    } else {
        quote! {}
    };

    // Generate state provider methods
    // 1. Generate __provide_contexts for nested enums (those WITHOUT state_suffix)
    // 2. Generate provide_state_contexts for root enum (one WITH state_suffix)
    let nested_provide_method = if state_store_type.is_none() {
        generate_nested_provide_method(&enum_ident, data)
    } else {
        quote! {}
    };

    let root_provide_method = if let Some(ref state_store_type) = state_store_type {
        generate_root_provide_method(&enum_ident, data, state_store_type)
    } else {
        quote! {}
    };

    let routable_impl = quote! {
        // Compile-time validation of state fields
        #field_validation

        // Generate context helper methods
        #context_helpers

        // Generate state provider methods
        #nested_provide_method
        #root_provide_method

        /* -----------------------------------------------------------------------------------------
         * `Routable` implementation
         * ---------------------------------------------------------------------------------------*/
        impl Routable for #enum_ident {

            /* -------------------------------------------------------------------------------------
             * `Routes` implementation
             * -----------------------------------------------------------------------------------*/
            fn routes() -> impl ::leptos::IntoView {
                #state_init

                ::leptos_router::components::Routes(
                    ::leptos_router::components::RoutesProps::builder()
                        .transition(#transition)
                        .fallback(#fallback)
                        .children(
                            ::leptos::children::ToChildren::to_children(move || {
                                (#(#children),*)
                            })
                        )
                        .build()
                )
            }

            /* -------------------------------------------------------------------------------------
             * `FlatRoutes` implementation
             * -----------------------------------------------------------------------------------*/
            fn flat_routes() -> impl ::leptos::IntoView {
                #state_init

                ::leptos_router::components::FlatRoutes(
                    ::leptos_router::components::FlatRoutesProps::builder()
                        .transition(#transition)
                        .fallback(#fallback)
                        .children(
                            ::leptos::children::ToChildren::to_children(move || {
                                (#(#children),*)
                            })
                        )
                        .build()
                )
            }

            /* -------------------------------------------------------------------------------------
             * `Fallback` implementation
             * -----------------------------------------------------------------------------------*/
            fn fallback() -> impl ::leptos::IntoView {
                #fallback
            }

            /* -------------------------------------------------------------------------------------
             * `ParentRoute` implementation
             * -----------------------------------------------------------------------------------*/
            fn parent_route<
                Path,
                View,
            >(
                path: Path,
                view: View,
                ssr: ::leptos_router::SsrMode,
            ) -> impl ::leptos_router::MatchNestedRoutes + Clone
            where
                Path: Send
                    + Sync
                    + 'static
                    + Clone
                    + std::fmt::Debug
                    + ::leptos_router::PossibleRouteMatch,
                View: ::leptos_router::ChooseView,
            {
                ::leptos_router::components::ParentRoute(
                    ::leptos_router::components::ParentRouteProps::builder()
                        .path(path)
                        .view(view)
                        .ssr(ssr)
                        .children(
                            ::leptos::children::ToChildren::to_children(move || {
                                (#(#children),*)
                            })
                        )
                        .build()
                )
            }

            /* -------------------------------------------------------------------------------------
             * `ProtectedParentRoute` implementation
             * -----------------------------------------------------------------------------------*/
            fn protected_parent_route<
                Path,
                View,
                ViewFn,
                ConditionFn,
                RedirectPathFn,
                RedirectPath,
            >(
                path: Path,
                view: ViewFn,
                condition: ConditionFn,
                fallback: ::leptos::children::ViewFn,
                redirect_path: RedirectPathFn,
                ssr: ::leptos_router::SsrMode,
            ) -> impl ::leptos_router::MatchNestedRoutes + Clone
            where
                Path: Send
                    + Sync
                    + 'static
                    + Clone
                    + std::fmt::Debug
                    + ::leptos_router::PossibleRouteMatch,
                ViewFn: Fn() -> View + Send + Clone + 'static,
                View: ::leptos::IntoView + 'static,
                ConditionFn: Fn() -> Option<bool> + Send + Clone + 'static,
                RedirectPathFn: Fn() -> RedirectPath + Send + Clone + 'static,
                RedirectPath: ::std::fmt::Display + 'static,
            {
                ::leptos_router::components::ProtectedParentRoute(
                    ::leptos_router::components::ProtectedParentRouteProps::builder()
                        .path(path)
                        .view(view)
                        .condition(condition)
                        .fallback(fallback)
                        .redirect_path(redirect_path)
                        .children(
                            ::leptos::children::ToChildren::to_children(move || {
                                (#(#children),*)
                            })
                        )
                        .ssr(ssr)
                        .build()
                )
            }
        }
    };

    let to_href_display_impl = match crate::to_href_display::generate_to_href_display_impl(&enum_ident, data) {
        Ok(ts) => ts,
        Err(e) => return e.to_compile_error().into(),
    };

    let from_str_impl = match generate_from_str_impl(&enum_ident, data) {
        Ok(ts) => ts,
        Err(e) => return e.to_compile_error().into(),
    };

    let from_asref_str_impl = generate_from_asref_str_impl(&enum_ident, data);

    let expanded = quote! {
        #routable_impl
        #to_href_display_impl
        #from_str_impl
        #from_asref_str_impl
    };
    expanded.into()
}

/* -------------------------------------------------------------------------------------------------
 * FromStr Implementation
 * -----------------------------------------------------------------------------------------------*/
fn generate_from_str_impl(
    enum_ident: &syn::Ident,
    data: &syn::DataEnum,
) -> syn::Result<proc_macro2::TokenStream> {
    let mut match_arms = Vec::new();

    for variant in &data.variants {
        let variant_ident = &variant.ident;
        let route_path = match crate::to_href_display::find_route_path(&variant.attrs) {
            Some(p) if !p.is_empty() => p,
            _ => {
                // Handle nested routers (single unnamed field)
                if let Fields::Unnamed(unnamed) = &variant.fields {
                    if unnamed.unnamed.len() == 1 {
                        let field_ty = &unnamed.unnamed[0].ty;
                        match_arms.push(quote! {
                            // Try nested route parsing
                            if let Ok(nested) = <#field_ty as ::std::str::FromStr>::from_str(input) {
                                return Ok(#enum_ident::#variant_ident(nested));
                            }
                        });
                    }
                }
                continue;
            }
        };

        let segments = crate::to_href_display::parse_segments(&route_path);
        let pattern_match = generate_pattern_match(&segments, &variant.fields, enum_ident, variant_ident)?;
        match_arms.push(pattern_match);
    }

    let parse_url_parts = parse_url_parts_tokens();

    Ok(quote! {
        impl ::std::str::FromStr for #enum_ident {
            type Err = String;

            fn from_str(input: &str) -> Result<Self, Self::Err> {
                #parse_url_parts

                // Parse URL to get path and query params
                let (path, query_params) = parse_url_parts(input);
                let path_segments: Vec<&str> = path.trim_start_matches('/')
                    .split('/')
                    .filter(|s| !s.is_empty())
                    .collect();

                #(#match_arms)*

                Err(format!("No route matches path: {}", input))
            }
        }
    })
}

/* -------------------------------------------------------------------------------------------------
 * From<AsRef<str>> Implementation (with fallback)
 * -----------------------------------------------------------------------------------------------*/
fn generate_from_asref_str_impl(
    enum_ident: &syn::Ident,
    data: &syn::DataEnum,
) -> proc_macro2::TokenStream {
    // Find the fallback variant
    let fallback_variant = data.variants.iter()
        .find(|v| v.attrs.iter().any(|attr| attr.path().is_ident("fallback")))
        .map(|v| &v.ident);

    let Some(fallback_ident) = fallback_variant else {
        // No fallback, don't generate From impl
        return quote!();
    };

    quote! {
        impl<T: AsRef<str>> From<T> for #enum_ident {
            fn from(value: T) -> Self {
                match <#enum_ident as ::std::str::FromStr>::from_str(value.as_ref()) {
                    Ok(route) => route,
                    Err(_) => #enum_ident::#fallback_ident,
                }
            }
        }
    }
}

/* -------------------------------------------------------------------------------------------------
 * Helper functions for FromStr
 * -----------------------------------------------------------------------------------------------*/
fn generate_pattern_match(
    segments: &[crate::to_href_display::RouteSegment],
    fields: &Fields,
    enum_ident: &syn::Ident,
    variant_ident: &syn::Ident,
) -> syn::Result<proc_macro2::TokenStream> {
    use crate::to_href_display::RouteSegment;

    let mut field_parsers = Vec::new();
    let mut required_segments = 0;
    let mut has_optional = false;

    // Count required segments and check for optional params
    for seg in segments {
        match seg {
            RouteSegment::Static(_) | RouteSegment::Param(_) => required_segments += 1,
            RouteSegment::OptionalParam(_) => has_optional = true,
        }
    }

    // Generate segment matching logic
    let mut segment_checks = Vec::new();
    let mut segment_idx = 0;

    for seg in segments {
        let idx = syn::Index::from(segment_idx);
        match seg {
            RouteSegment::Static(text) => {
                segment_checks.push(quote! {
                    if path_segments.get(#idx) != Some(&#text) {
                        return false;
                    }
                });
                segment_idx += 1;
            }
            RouteSegment::Param(name) => {
                let field_ident = syn::Ident::new(name, proc_macro2::Span::call_site());
                field_parsers.push(quote! {
                    let #field_ident = path_segments[#idx]
                        .parse()
                        .map_err(|_| format!("Failed to parse {} as expected type", #name))?;
                });
                segment_idx += 1;
            }
            RouteSegment::OptionalParam(name) => {
                let field_ident = syn::Ident::new(name, proc_macro2::Span::call_site());
                field_parsers.push(quote! {
                    let #field_ident = path_segments.get(#idx)
                        .and_then(|s| s.parse().ok());
                });
                segment_idx += 1;
            }
        }
    }

    // Handle query parameters for optional fields
    let query_param_parsers = generate_query_param_parsers(fields, segments);

    // Get nested field type if this is a parent route with nested routes
    let nested_field_ty = if let Fields::Unnamed(unnamed) = fields {
        if unnamed.unnamed.len() == 1 {
            Some(&unnamed.unnamed[0].ty)
        } else {
            None
        }
    } else {
        None
    };

    // Build the variant constructor
    let variant_constructor = build_variant_constructor(enum_ident, variant_ident, fields, segments, nested_field_ty)?;

    // Build complete matching logic
    let max_segments_val = syn::Index::from(segment_idx);
    let required_segments_val = syn::Index::from(required_segments);
    let segment_count_val = syn::Index::from(segment_idx);

    // For nested routes, allow more segments than the parent path
    let max_segments = if nested_field_ty.is_some() {
        quote! { path_segments.len() >= #required_segments_val }
    } else if has_optional {
        quote! { path_segments.len() <= #max_segments_val }
    } else {
        quote! { path_segments.len() == #required_segments_val }
    };

    // For nested routes, include the parsing logic before the constructor
    if nested_field_ty.is_some() {
        Ok(quote! {
            // Check if this route matches
            let matches = || -> bool {
                if path_segments.len() < #required_segments_val {
                    return false;
                }
                if !(#max_segments) {
                    return false;
                }
                #(#segment_checks)*
                true
            };

            if matches() {
                let segment_count = #segment_count_val;
                #(#field_parsers)*
                #query_param_parsers
                #variant_constructor
            }
        })
    } else {
        Ok(quote! {
            // Check if this route matches
            let matches = || -> bool {
                if path_segments.len() < #required_segments_val {
                    return false;
                }
                if !(#max_segments) {
                    return false;
                }
                #(#segment_checks)*
                true
            };

            if matches() {
                let segment_count = #segment_count_val;
                #(#field_parsers)*
                #query_param_parsers
                return Ok(#variant_constructor);
            }
        })
    }
}

fn generate_query_param_parsers(
    fields: &Fields,
    segments: &[crate::to_href_display::RouteSegment],
) -> proc_macro2::TokenStream {
    // Collect field names used in path
    let mut used_fields = std::collections::HashSet::new();
    for seg in segments {
        match seg {
            crate::to_href_display::RouteSegment::Param(name) |
            crate::to_href_display::RouteSegment::OptionalParam(name) => {
                used_fields.insert(name.clone());
            }
            _ => {}
        }
    }

    let mut parsers = Vec::new();

    if let Fields::Named(named) = fields {
        for field in &named.named {
            let field_name = field.ident.as_ref().unwrap();
            let field_name_str = field_name.to_string();

            // Skip fields already parsed from path
            if used_fields.contains(&field_name_str) {
                continue;
            }

            // Only handle Option fields in query params
            if crate::to_href_display::is_option_type(&field.ty) {
                parsers.push(quote! {
                    let #field_name = query_params.get(#field_name_str)
                        .and_then(|v| v.parse().ok());
                });
            }
        }
    }

    quote! { #(#parsers)* }
}

fn build_variant_constructor(
    enum_ident: &syn::Ident,
    variant_ident: &syn::Ident,
    fields: &Fields,
    segments: &[crate::to_href_display::RouteSegment],
    nested_field_ty: Option<&syn::Type>,
) -> syn::Result<proc_macro2::TokenStream> {
    match fields {
        Fields::Unit => Ok(quote! { #enum_ident::#variant_ident }),
        Fields::Named(named) => {
            let mut field_inits = Vec::new();

            for field in &named.named {
                let field_name = field.ident.as_ref().unwrap();
                let field_name_str = field_name.to_string();

                // Check if field is used in path
                let in_path = segments.iter().any(|seg| match seg {
                    crate::to_href_display::RouteSegment::Param(name) |
                    crate::to_href_display::RouteSegment::OptionalParam(name) => name == &field_name_str,
                    _ => false,
                });

                if in_path {
                    field_inits.push(quote! { #field_name });
                } else if crate::to_href_display::is_option_type(&field.ty) {
                    // Query param field (should be Option)
                    field_inits.push(quote! { #field_name });
                } else {
                    // Non-Option field not in path - this shouldn't happen with proper validation
                    field_inits.push(quote! { #field_name: Default::default() });
                }
            }

            Ok(quote! { #enum_ident::#variant_ident { #(#field_inits),* } })
        }
        Fields::Unnamed(unnamed) => {
            if unnamed.unnamed.len() == 1 {
                // For nested routes (parent routes), we need to parse the remaining path
                if let Some(field_ty) = nested_field_ty {
                    Ok(quote! {
                        {
                            // Construct the remaining path for nested route
                            let remaining_path = if path_segments.len() > segment_count {
                                let remaining: Vec<&str> = path_segments[segment_count..].to_vec();
                                format!("/{}", remaining.join("/"))
                            } else {
                                "/".to_string()
                            };

                            // Parse nested route using FromStr
                            let nested = <#field_ty as ::std::str::FromStr>::from_str(&remaining_path)
                                .map_err(|_| format!("Failed to parse nested route at path: {}", input))?;

                            return Ok(#enum_ident::#variant_ident(nested));
                        }
                    })
                } else {
                    Err(syn::Error::new(
                        variant_ident.span(),
                        "Nested route field type not provided",
                    ))
                }
            } else {
                Err(syn::Error::new(
                    variant_ident.span(),
                    "Variants with multiple unnamed fields are not supported",
                ))
            }
        }
    }
}

// Helper function to parse URL into path and query params
fn parse_url_parts_tokens() -> proc_macro2::TokenStream {
    quote! {
        fn parse_url_parts(url: &str) -> (&str, std::collections::HashMap<String, String>) {
            let mut query_params = std::collections::HashMap::new();

            let (path, query) = if let Some(idx) = url.find('?') {
                (&url[..idx], Some(&url[idx + 1..]))
            } else {
                (url, None)
            };

            if let Some(query_str) = query {
                for pair in query_str.split('&') {
                    if let Some(eq_idx) = pair.find('=') {
                        let key = &pair[..eq_idx];
                        let value = &pair[eq_idx + 1..];
                        query_params.insert(key.to_string(), value.to_string());
                    }
                }
            }

            (path, query_params)
        }
    }
}

fn generate_nested_provide_method(
    enum_ident: &syn::Ident,
    data: &syn::DataEnum,
) -> TokenStream2 {
    let provide_statements = generate_recursive_provides(data, quote! { parent_sub_state });

    let enum_name = enum_ident.to_string();
    let base_name = if enum_name.ends_with("Routes") {
        enum_name.trim_end_matches("Routes")
    } else {
        &enum_name
    };

    let sub_state_type = syn::Ident::new(
        &format!("{}SubState", base_name),
        enum_ident.span()
    );

    let trait_name = syn::Ident::new(
        &format!("{}StoreFields", sub_state_type),
        enum_ident.span()
    );

    quote! {
        impl #enum_ident {
            #[doc(hidden)]
            pub fn __provide_contexts<F>(parent_sub_state: F)
            where
                F: reactive_stores::StoreField<Value = #sub_state_type> + #trait_name<F> + Clone + Send + Sync + 'static,
            {
                #(#provide_statements)*
            }
        }
    }
}

fn generate_root_provide_method(
    enum_ident: &syn::Ident,
    data: &syn::DataEnum,
    state_store_type: &syn::Ident,
) -> TokenStream2 {
    let provide_statements = generate_recursive_provides(data, quote! { root_store });

    quote! {
        impl #enum_ident {
            pub fn provide_state_contexts(root_store: reactive_stores::Store<#state_store_type>) {
                leptos::prelude::provide_context(root_store.clone());
                #(#provide_statements)*
            }
        }
    }
}

/// Recursively generate provide_context statements for a route enum and all nested enums
fn generate_recursive_provides(
    data: &syn::DataEnum,
    accessor: TokenStream2,
) -> Vec<TokenStream2> {
    let mut statements = Vec::new();

    for variant in &data.variants {
        let field_name = syn::Ident::new(
            &to_snake_case(&variant.ident.to_string()),
            variant.ident.span()
        );

        statements.push(quote! {
            leptos::prelude::provide_context(#accessor.clone().#field_name());
        });

        if let syn::Fields::Unnamed(fields) = &variant.fields {
            if let Some(syn::Field { ty: syn::Type::Path(type_path), .. }) = fields.unnamed.first() {
                if let Some(nested_enum) = type_path.path.segments.last() {
                    let nested_enum_ident = &nested_enum.ident;

                    statements.push(quote! {
                        leptos::prelude::provide_context(#accessor.clone().#field_name().sub_state());
                    });

                    statements.push(quote! {
                        #nested_enum_ident::__provide_contexts(#accessor.clone().#field_name().sub_state());
                    });
                }
            }
        }
    }

    statements
}

/* -------------------------------------------------------------------------------------------------
 * Parse Route Kind
 * -----------------------------------------------------------------------------------------------*/
fn parse_variant(variant: &syn::Variant) -> Result<Option<RouteKind>, darling::Error> {
    Ok(try_parse_variants!(
        variant,
        RouteVariant,
        ParentRouteVariant,
        ProtectedRouteVariant,
        ProtectedParentRouteVariant
    ))
}

fn multiple_route_error(variant: &syn::Variant) -> darling::Error {
    syn::Error::new(
        variant.span(),
        "Multiple route-like attributes found. Only one of `#[route]`, `#[parent_route]`, \
         `#[protected_route]`, or `#[protected_parent_route]` is allowed.",
    ).into()
}

#[allow(unused)]
fn parse_dynamic_segments(path: &str, variant: &Variant) -> Vec<(String, Box<Type>)> {
    let field_types = collect_named_fields(variant);
    let mut segments = Vec::new();
    for seg in path.split('/') {
        if let Some(field_name) = seg.strip_prefix(':') {
            let field_name = field_name.trim().to_string();
            if !field_name.is_empty() {
                let ty = field_types
                    .get(&field_name)
                    .cloned()
                    .unwrap_or_else(|| Box::new(syn::parse_str("String").unwrap()));
                segments.push((field_name, ty));
            }
        }
    }
    segments
}

fn collect_named_fields(variant: &Variant) -> std::collections::HashMap<String, Box<Type>> {
    let mut map = std::collections::HashMap::new();
    if let Fields::Named(named_fields) = &variant.fields {
        for field in &named_fields.named {
            if let Some(ident) = &field.ident {
                map.insert(ident.to_string(), Box::new(field.ty.clone()));
            }
        }
    }
    map
}

fn parse_fallback_attrs(
    variant: &syn::Variant,
    _input_span: Span2,
    _fallback_func: &Option<TokenStream2>,
) -> syn::Result<()> {
    let fallback_attrs: Vec<_> = variant
        .attrs
        .iter()
        .filter(|attr| attr.path().is_ident("fallback"))
        .collect();

    if fallback_attrs.is_empty() {
        return Ok(());
    }
    if fallback_attrs.len() > 1 {
        return Err(syn::Error::new(
            variant.span(),
            "Multiple `#[fallback(...)]` attributes found on the same variant.",
        ));
    }

    // validate_single_fallback(input_span, fallback_func)?;
    Ok(())
}

#[allow(unused)]
fn validate_single_fallback(
    input_span: Span2,
    fallback_func: &Option<TokenStream2>,
) -> syn::Result<()> {
    if fallback_func.is_some() {
        return Err(syn::Error::new(
            input_span,
            "`#[fallback]` may only be set on one variant.",
        ));
    }
    Ok(())
}
