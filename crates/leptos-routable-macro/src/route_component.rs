//! Implements `#[route_component(...)]` on a function, creating a “hooking” function.
use proc_macro::TokenStream;
use quote::quote;
use syn::{
    spanned::Spanned,
    ItemFn, FnArg, PatType, Path,
};
use deluxe::Error;

/// Internal struct for the macro attribute (e.g. `#[route_component(AppRoute::Foo)]`).
struct RouteComponentAttr {
    variant_path: Option<Path>,
}

impl syn::parse::Parse for RouteComponentAttr {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.is_empty() {
            Ok(Self { variant_path: None })
        } else {
            let p: Path = input.parse()?;
            Ok(Self { variant_path: Some(p) })
        }
    }
}

/// Helper for building hooking function names: `"__ROUTE_COMP_{Enum}_{Variant}"`
fn build_registry_func_name(fn_name: &str) -> syn::Ident {
    let prefix = "__ROUTE_COMP_";
    let name = format!("{}{}", prefix, fn_name);
    syn::Ident::new(&name, fn_name.span())
}

/// The core implementation of `#[route_component(...)]`.
pub fn route_component_impl(attr: TokenStream, item: TokenStream) -> TokenStream {
    // 1) Parse the attribute for e.g. `AppRoute::Posts`
    let route_args = match syn::parse::<RouteComponentAttr>(attr) {
        Ok(v) => v,
        Err(e) => return e.to_compile_error().into(),
    };

    // 2) Parse the user’s function
    let input_ast = match syn::parse::<ItemFn>(item) {
        Ok(v) => v,
        Err(e) => return e.to_compile_error().into(),
    };
    let fn_name = &input_ast.sig.ident;
    let fn_vis = &input_ast.vis;
    let fn_generics = &input_ast.sig.generics;
    let fn_body = &input_ast.block;

    // Must specify something like `#[route_component(AppRoute::Posts)]`
    let Some(variant_path) = route_args.variant_path else {
        return Error::new(
            fn_name.span(),
            "`#[route_component(...)]` must specify a variant, e.g. `AppRoute::Posts`",
        )
            .to_compile_error()
            .into();
    };

    // Build hooking function name: "__ROUTE_COMP_AppRoute_Posts"
    let segments: Vec<String> = variant_path
        .segments
        .iter()
        .map(|seg| seg.ident.to_string())
        .collect();
    let enum_variant_str = segments.join("_");
    let registry_func_name = build_registry_func_name(&enum_variant_str);

    // 3) Inspect each parameter; parse if `#[params]` or `#[query]`.
    let mut param_statements = Vec::new();
    let mut param_idents = Vec::new();

    let has_attr = |attrs: &Vec<syn::Attribute>, needle: &str| -> bool {
        attrs.iter().any(|attr| attr.path().is_ident(needle))
    };

    for arg in &input_ast.sig.inputs {
        if let FnArg::Typed(PatType { pat, ty, attrs, .. }) = arg {
            // The parameter’s name (e.g., for `foo: MyParams`, `pname` = "foo")
            let pname = if let syn::Pat::Ident(id) = &**pat {
                id.ident.clone()
            } else {
                return Error::new_spanned(
                    arg,
                    "Only simple identifiers in function parameters are supported.",
                )
                    .to_compile_error()
                    .into();
            };

            let is_params = has_attr(&attrs, "params");
            let is_query = has_attr(&attrs, "query");

            if is_params {
                // Use leptos_router’s typed `use_params::<T>()`
                param_statements.push(quote! {
                    let #pname = ::leptos_router::hooks::use_params::<#ty>();
                });
            } else if is_query {
                // Use leptos_router’s typed `use_query::<T>()`
                param_statements.push(quote! {
                    let #pname = ::leptos_router::hooks::use_query::<#ty>();
                });
            } else {
                param_statements.push(quote! {
                    compile_error!("Parameter is not annotated with #[params] or #[query].")
                });
            }

            param_idents.push(pname);
        } else {
            return Error::new_spanned(
                arg,
                "Expected a typed parameter (e.g. `id: u32`).",
            )
                .to_compile_error()
                .into();
        }
    }

    // 4) Build the hooking function. This is what `<Route>.view(...)` calls:
    let hooking_func = quote! {
        #[::leptos::prelude::component]
        pub fn #registry_func_name() -> impl ::leptos::prelude::IntoView {
            #(#param_statements)*
            #fn_name(#(#param_idents),*)
        }
    };

    // 5) Keep the user’s original function as-is
    let param_tokens = input_ast
        .sig
        .inputs
        .iter()
        .map(|arg| quote! { #arg });

    let original_fn = quote! {
        #[allow(non_snake_case)]
        #fn_vis fn #fn_name #fn_generics (#(#param_tokens),*) -> impl ::leptos::IntoView {
            #fn_body
        }
    };

    // 6) Combine & return
    let expanded = quote! {
        // The original user function
        #original_fn

        // Our hooking function
        #hooking_func
    };

    expanded.into()
}
