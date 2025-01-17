use std::fmt::Debug;
mod maybe_param;
mod combine_paths;

pub trait Routable {
    fn routes() -> impl ::leptos::IntoView;

    fn flat_routes() -> impl ::leptos::IntoView;

    fn fallback() -> impl ::leptos::IntoView;

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
        + Debug
        + ::leptos_router::PossibleRouteMatch,
        View: ::leptos_router::ChooseView;

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
        + Debug
        + ::leptos_router::PossibleRouteMatch,
        ViewFn: Fn() -> View + Send + Clone + 'static,
        View: ::leptos::IntoView + 'static,
        ConditionFn: Fn() -> Option<bool> + Send + Clone + 'static,
        RedirectPathFn: Fn() -> RedirectPath + Send + Clone + 'static,
        RedirectPath: ::std::fmt::Display + 'static;
}


pub mod prelude {
    pub use leptos_routable_macro::*;
    pub use crate::maybe_param::*;
    pub use super::Routable;
    pub use super::combine_paths::combine_paths;
}
