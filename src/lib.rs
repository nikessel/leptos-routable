#[cfg(feature = "hook-extensions")]
mod hook_extensions;
mod maybe_param;

pub mod prelude {
    #[cfg(feature = "hook-extensions")]
    pub use super::hook_extensions::*;
    pub use leptos_routable_macro::*;
    pub use crate::maybe_param::*;
}

/// A simple trait that returns a dynamic `String` path at runtime,
/// embedding variant fields into any `:param` or `:param?`.
pub trait ToPath {
    fn to_path(&self) -> String;
}
