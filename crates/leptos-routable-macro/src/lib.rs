#![allow(clippy::needless_return)]
extern crate proc_macro;
pub(crate) mod derive_routable;
pub(crate) mod derive_to_href;
pub(crate) mod utils;

#[proc_macro_derive(Routable, attributes(route, fallback, routes, protected_route, parent_route, protected_parent_route))]
pub fn derive_routable(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    derive_routable::derive_routable_impl(input)
}

#[proc_macro_derive(ToHref, attributes(route))]
pub fn derive_to_path(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    derive_to_href::derive_to_href_impl(input)
}
