#[cfg(feature="hook-extensions")]
mod hook_extensions;

pub mod prelude {
    #[cfg(feature="hook-extensions")]
    pub use super::hook_extensions::*;
    pub use leptos_routable_macro::*;
}


