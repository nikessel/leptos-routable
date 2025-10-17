use syn::{parse2, File};
use proc_macro2::{TokenStream as TokenStream2};
use prettyplease::unparse;

#[allow(unused)]
pub(crate) fn format_generated_code(expanded: TokenStream2) -> TokenStream2 {
    match parse2::<File>(expanded.clone()) {
        Ok(file) => {
            let formatted_code = unparse(&file);
            formatted_code.parse().unwrap_or(expanded)
        }
        Err(_) => expanded,
    }
}

pub(crate) fn build_variant_view_name(
    _enum_ident: &syn::Ident,
    variant_ident: &syn::Ident,
    config: &crate::derive_routable::RoutableConfiguration
) -> syn::Ident {
    let name = variant_ident.to_string();

    // Add prefix and suffix
    let full_name = format!(
        "{}{}{}",
        config.view_prefix.clone(),
        name,
        config.view_suffix
    );

    // Convert to syn::Ident, preserving the original span
    syn::Ident::new(&full_name, variant_ident.span())
}

/// Converts a PascalCase identifier to snake_case
fn to_snake_case(s: &str) -> String {
    let mut result = String::new();
    let mut chars = s.chars().peekable();

    while let Some(c) = chars.next() {
        if c.is_uppercase() {
            // Add underscore before uppercase letter if:
            // 1. We're not at the start
            // 2. The next char is lowercase (avoid turning "URLPath" into "u_r_l_path")
            if !result.is_empty() {
                if let Some(&next) = chars.peek() {
                    if next.is_lowercase() {
                        result.push('_');
                    }
                }
            }
            result.extend(c.to_lowercase());
        } else {
            result.push(c);
        }
    }

    result
}

/// Builds a module path for a route view component
///
/// For example, with module_prefix = "routes", variant_ident = "Dashboard", is_parent = true:
/// Returns: `crate::routes::dashboard::layout::Layout`
pub(crate) fn build_module_view_path(
    variant_ident: &syn::Ident,
    is_parent: bool,
    module_prefix: &str,
) -> TokenStream2 {
    let module_name = to_snake_case(&variant_ident.to_string());
    let view_or_layout = if is_parent { "layout" } else { "view" };
    let component_name = if is_parent { "Layout" } else { "View" };

    // Convert path separators (/) to Rust module separators (::)
    let module_prefix_normalized = module_prefix.replace('/', "::");

    let path_tokens: TokenStream2 = format!(
        "crate::{}::{}::{}::{}",
        module_prefix_normalized,
        module_name,
        view_or_layout,
        component_name
    ).parse().unwrap();

    path_tokens
}
