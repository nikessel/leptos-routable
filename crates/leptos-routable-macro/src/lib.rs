#![allow(clippy::needless_return)]
extern crate proc_macro;

use proc_macro::TokenStream;
mod derive_routable;
mod route_component;

// Public macro entry point for `#[derive(Routable)]`
#[proc_macro_derive(Routable, attributes(route, fallback))]
pub fn derive_routable(input: TokenStream) -> TokenStream {
    derive_routable::derive_routable_impl(input)
}

// Public macro entry point for `#[route_component(...)]`
#[proc_macro_attribute]
pub fn route_component(attr: TokenStream, item: TokenStream) -> TokenStream {
    route_component::route_component_impl(attr, item)
}
