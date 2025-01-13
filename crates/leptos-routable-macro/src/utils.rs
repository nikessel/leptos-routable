use proc_macro2::TokenStream as TokenStream2;
use prettyplease::unparse;
use syn::{parse2, spanned::Spanned, File, Ident};

/// Builds a hooking function name of the form `"__ROUTE_COMP_{Enum}_{Variant}"`.
pub(crate) fn build_registry_func_name(fn_name: &str) -> Ident {
    let prefix = "__ROUTE_COMP_";
    let full_name = format!("{}{}", prefix, fn_name);
    Ident::new(&full_name, fn_name.span())
}

/// Attempts to format the provided token stream as well-formed Rust code.
pub(crate) fn format_generated_code(expanded: TokenStream2) -> TokenStream2 {
    match parse2::<File>(expanded.clone()) {
        Ok(file) => {
            let formatted_code = unparse(&file);
            formatted_code.parse().unwrap_or(expanded)
        }
        Err(_) => expanded,
    }
}
