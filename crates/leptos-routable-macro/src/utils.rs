use syn::{parse2, spanned::Spanned, File};
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
    let mut name = variant_ident.to_string();

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