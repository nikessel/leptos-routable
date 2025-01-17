use proc_macro::TokenStream;
use proc_macro2::{Span as Span2, TokenStream as TokenStream2};
use quote::quote;
use syn::{parse_macro_input, spanned::Spanned, Data::{Enum, Struct, Union}, DeriveInput, Ident, Type, Variant, Fields};
use darling::{FromDeriveInput, FromVariant};

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
    #[allow(unused)]
    attrs: Vec<syn::Attribute>,

    #[darling(default)]
    pub(crate) transition: bool,

    #[darling(default)]
    pub(crate) view_prefix: String,

    #[darling(default = "default_view_suffix")]
    pub(crate) view_suffix: String,
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

        if let Some(kind) = route_kind {
            if let Some(child_ts) = kind.into_child_tokens(view_ident) {
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
    let routable_impl = quote! {
        /* -----------------------------------------------------------------------------------------
         * `Routable` implementation
         * ---------------------------------------------------------------------------------------*/
        impl Routable for #enum_ident {

            /* -------------------------------------------------------------------------------------
             * `FlatRoutes` implementation
             * -----------------------------------------------------------------------------------*/
            fn routes() -> impl ::leptos::IntoView {
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

    let expanded = quote! {
        #routable_impl
        #to_href_display_impl
    };
    expanded.into()
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
