use leptos::prelude::*;
use leptos_meta::{Html, Meta, Title};
use leptos_routable::prelude::*;
use leptos_router::params::Params;

#[derive(Routable)]
pub enum AppRouter {
    #[route(path = "/")]
    Home,
    #[route(path = "/contact")]
    Contact,
    #[route(path = "/asset")]
    AssetList,
    #[route(path = "/asset/:id")]
    AssetDetails { id: u64 },
    #[fallback]
    #[route(path = "/404")]
    NotFound,
}

#[route_component(AppRouter::Home)]
pub fn HomeView() -> impl IntoView {
    view! {
        <h1>"Welcome Home!"</h1>
    }
}

#[route_component(AppRouter::Contact)]
pub fn ContactView() -> impl IntoView {
    view! {
        <h1>"Contact Us"</h1>
    }
}

#[derive(Params, PartialEq, Debug)]
pub struct MyQuery {
    pub q: Option<String>,
}

#[route_component(AppRouter::AssetDetails)]
pub fn AssetDetailsView() -> impl IntoView {
    view! {
        <h1>"Asset Details"</h1>
    }
}

#[route_component(AppRouter::AssetList)]
pub fn AssetListView() -> impl IntoView {
    view! {
        <h1>"Asset List"</h1>
    }
}

#[route_component(AppRouter::NotFound)]
pub fn FallbackView() -> impl IntoView {
    view! {
        <h1>"404"</h1>
    }
}

#[component]
pub fn App() -> impl IntoView {
    leptos_meta::provide_meta_context();
    view! {
        <Html attr:lang="en" attr:dir="ltr" attr:data-theme="light" />
        <Title text="Welcome to Leptos CSR" />
        <Meta charset="UTF-8" />
        <Meta name="viewport" content="width=device-width, initial-scale=1.0" />
        <AppRouter/>
    }
}
