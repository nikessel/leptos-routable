use std::marker::PhantomData;
use leptos::prelude::{component, *};
use leptos_meta::{Html, Meta, Title};
use leptos_router::params::{Params, ParamsError, ParamsMap};
use leptos_routable::prelude::*;

#[derive(Routable)]
#[routing(mode = "flat")]
pub enum AppRouter {
    #[route(path = "/")]
    Home,
    #[route(path = "/contact")]
    Contact,
    #[route(path = "/asset")]
    AssetList,
    #[route(path = "/asset/:id")]  // TODO Optional params
    AssetDetails { id: u64, action: String },
    #[fallback(replace)]
    #[route(path = "/404")]
    NotFound,
}

#[derive(Params, PartialEq, Debug)]
pub struct AssetQuery {
    pub q: Option<String>,
    pub lang: Option<String>,
}

#[route_component(AppRouter::Home)]
pub fn HomeView() -> impl IntoView {
    view! {
        // Minimal styling for the demo
        <div class="p-4 text-center">
            <h1 class="text-2xl font-bold">"Welcome Home!"</h1>
        </div>
    }
}

use leptos_router::hooks::use_params;

#[route_component(AppRouter::Contact)]
pub fn ContactView() -> impl IntoView {
    view! {
        <div class="p-4 text-center">
            <h1 class="text-2xl font-bold">"Contact Us"</h1>
        </div>
    }
}

use leptos::*;
use leptos_router::NavigateOptions;
use leptos_router::hooks::use_navigate;
use leptos_router::components::A;

#[route_component(AppRouter::AssetDetails)]
pub fn AssetDetailsView(
    #[param] param: Memo<ParamsMap>,
    #[query] query: Memo<ParamsMap>,
) -> impl IntoView {
    let id = Memo::new(
        move |_| param
            .get()
            .get("id")
            .unwrap_or_default()
            .parse::<u64>()
            .unwrap()
    );

    view! {
        <div class="flex flex-col items-center p-4 space-y-4">
            <h1 class="text-2xl font-bold">
                { move || format!("Asset ID: {}", id.get()) }
            </h1>
            <h2>
                { move || format!("{}", query.get().to_query_string()) }
            </h2>

            <div class="flex space-x-4">
                <A
                    href="/"
                    attr:class="px-4 py-2 bg-green-500 text-white rounded"
                >
                    {"Home (A)"}
                </A>
                <A
                    href=move || format!("/asset/{}", id.get() - 1)
                    attr:class="px-4 py-2 bg-blue-500 text-white rounded disabled:opacity-50"
                    attr:disabled=move || id.get() <= 1
                >
                    {"Previous (A)"}
                </A>
                <A
                    href=move || format!("/asset/{}", id.get() + 1)
                    attr:class="px-4 py-2 bg-blue-500 text-white rounded"
                >
                    {"Next (A)"}
                </A>
            </div>
        </div>
    }
    // ()
}

#[route_component(AppRouter::AssetList)]
pub fn AssetListView() -> impl IntoView {
    view! {
        <div class="p-4 text-center">
            <h1 class="text-2xl font-bold">"Asset List"</h1>
        </div>
    }
}

#[route_component(AppRouter::NotFound)]
pub fn FallbackView() -> impl IntoView {
    view! {
        <div class="p-4 text-center">
            <h1 class="text-2xl font-bold">"404: Not Found"</h1>
        </div>
    }
}

#[component]
pub fn App() -> impl IntoView {
    leptos_meta::provide_meta_context();

    view! {
        <Html attr:lang="en" attr:dir="ltr" />
        <Title text="Welcome to Leptos CSR" />
        <Meta charset="UTF-8" />
        <Meta name="viewport" content="width=device-width, initial-scale=1.0" />

        // Just wrap everything in a minimal container
        <main class="min-h-screen">
            <AppRouter/>
        </main>
    }
}
