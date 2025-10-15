use leptos::prelude::{component, *};
use leptos_meta::{Html, Meta, Title};
use leptos_routable::prelude::*;
use leptos_router::hooks::{use_location};
use leptos_router::components::{Router, A};

#[derive(Routable, PartialEq, Debug)]
#[routes(
    view_prefix = "",
    view_suffix = "View",
    transition = false
)]
pub enum AppRoutes {
    #[route(path = "/")]
    Home,
    #[route(path = "/contact")]
    Contact,
    #[route(path = "/asset")]
    AssetList,
    #[route(path = "/asset/:id")]
    AssetDetails {
        id: u64,
        action: Option<String>,
    },
    #[route(path = "/profile")]
    Profile,
    #[fallback]
    #[route(path = "/404")]
    NotFound,
}

#[component]
pub fn HomeView() -> impl IntoView {
    view! {
        <div class="p-4 text-center">
            <h1 class="text-2xl font-bold">"Welcome Home!"</h1>
            <p>"Explore the site using the navigation links below."</p>
        </div>
    }
}

#[component]
pub fn ContactView() -> impl IntoView {
    view! {
        <div class="p-4 text-center">
            <h1 class="text-2xl font-bold">"Contact Us"</h1>
            <p>"Reach out at: contact@myapp.com"</p>
        </div>
    }
}

#[component]
pub fn AssetListView() -> impl IntoView {
    view! {
        <div class="p-4">
            <h1 class="text-2xl font-bold mb-4">"Asset List"</h1>
            <div class="space-y-4">
                <h2 class="text-xl">"Test Navigation Links"</h2>
                <div class="flex flex-col space-y-2">
                    <A
                        href=AppRoutes::Home
                        attr:class="inline-block px-4 py-2 bg-green-500 text-white rounded"
                    >
                        "→ Go Home"
                    </A>

                    <A
                        href=AppRoutes::Contact
                        attr:class="inline-block px-4 py-2 bg-blue-500 text-white rounded"
                    >
                        "→ Contact Page"
                    </A>

                    <A
                        href=AppRoutes::AssetDetails {
                            id: 123,
                            action: None,
                        }
                        attr:class="inline-block px-4 py-2 bg-blue-500 text-white rounded"
                    >
                        "→ Asset 123 (no action)"
                    </A>

                    <A
                        href=AppRoutes::AssetDetails {
                            id: 456,
                            action: Some("edit".to_string()),
                        }
                        attr:class="inline-block px-4 py-2 bg-blue-500 text-white rounded"
                    >
                        "→ Asset 456 (edit action)"
                    </A>

                    <A
                        href=AppRoutes::Profile
                        attr:class="inline-block px-4 py-2 bg-blue-500 text-white rounded"
                    >
                        "→ Profile Page"
                    </A>

                    <A
                        href=AppRoutes::NotFound
                        attr:class="inline-block px-4 py-2 bg-blue-500 text-white rounded"
                    >
                        "→ 404 Page"
                    </A>
                </div>

                <div class="mt-4 p-4 rounded">
                    <p class="text-sm font-mono">
                        "Current Path: " {move || use_location().pathname}
                    </p>
                    <p class="text-sm font-mono">
                        "Query String: " {move || use_location().search}
                    </p>
                </div>
            </div>
        </div>
    }
}

#[component]
pub fn AssetDetailsView() -> impl IntoView {
    let id = MaybeParam::<u64>::new("id").ok();

    let prev_href = move || {
        AppRoutes::AssetDetails {
            id: id.get().unwrap_or_default().saturating_sub(1),
            action: None,
        }
            .to_string()
    };

    let next_href = move || {
        AppRoutes::AssetDetails {
            id: id.get().unwrap_or_default() + 1,
            action: None,
        }
            .to_string()
    };

    view! {
        <div class="flex flex-col items-center p-4 space-y-4">
            <h1 class="text-2xl font-bold">
                {move || format!("Asset ID: {}", id.get().unwrap_or_default())}
            </h1>

            <div class="flex space-x-4">
                <A href=AppRoutes::Home attr:class="px-4 py-2 bg-green-500 text-white rounded">
                    "Home"
                </A>

                <A
                    href=prev_href
                    attr:class="px-4 py-2 bg-blue-500 text-white rounded disabled:opacity-50"
                    attr:disabled=move || id.get().unwrap_or_default() <= 1
                >
                    "Previous"
                </A>

                <A href=next_href attr:class="px-4 py-2 bg-blue-500 text-white rounded">
                    "Next"
                </A>
            </div>
        </div>
    }
}

#[component]
pub fn ProfileView() -> impl IntoView {
    view! {
        <div class="p-4 text-center">
            <h1 class="text-2xl font-bold">"User Profile"</h1>
            <p>"Name: John Doe"</p>
            <p>"Membership: Gold"</p>
            <p>"Email: john.doe@example.com"</p>
            <A
                href=AppRoutes::Home
                attr:class="inline-block px-4 py-2 mt-4 bg-green-500 text-white rounded"
            >
                "Back Home"
            </A>
        </div>
    }
}

#[component]
pub fn NotFoundView() -> impl IntoView {
    view! {
        <div class="p-4 text-center">
            <h1 class="text-2xl font-bold">"404: Not Found"</h1>
            <p>"Sorry, we can't find that page."</p>
            <A
                href=AppRoutes::Home
                attr:class="inline-block px-4 py-2 bg-green-500 text-white rounded mt-4"
            >
                "Go Home"
            </A>
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

        <main class="min-h-screen">
            <Router>
                <nav class="flex space-x-4 p-4 bg-gray-900 text-white">
                    <A href=AppRoutes::Home attr:class="text-white px-3 py-1 bg-green-600 rounded">
                        "Home"
                    </A>
                    <A
                        href=AppRoutes::Contact
                        attr:class="text-white px-3 py-1 bg-blue-600 rounded"
                    >
                        "Contact"
                    </A>
                    <A
                        href=AppRoutes::AssetList
                        attr:class="text-white px-3 py-1 bg-blue-600 rounded"
                    >
                        "Assets"
                    </A>
                    <A
                        href=AppRoutes::Profile
                        attr:class="text-white px-3 py-1 bg-blue-600 rounded"
                    >
                        "Profile"
                    </A>
                </nav>

                {move || AppRoutes::routes()}
            </Router>
        </main>
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_from_str_home() {
        let route = AppRoutes::from_str("/").unwrap();
        assert_eq!(route, AppRoutes::Home);
    }

    #[test]
    fn test_from_str_contact() {
        let route = AppRoutes::from_str("/contact").unwrap();
        assert_eq!(route, AppRoutes::Contact);
    }

    #[test]
    fn test_from_str_asset_with_id() {
        let route = AppRoutes::from_str("/asset/123").unwrap();
        match route {
            AppRoutes::AssetDetails { id, action } => {
                assert_eq!(id, 123);
                assert_eq!(action, None);
            }
            _ => panic!("Expected AssetDetails variant"),
        }
    }

    #[test]
    fn test_from_str_asset_with_query() {
        let route = AppRoutes::from_str("/asset/456?action=edit").unwrap();
        match route {
            AppRoutes::AssetDetails { id, action } => {
                assert_eq!(id, 456);
                assert_eq!(action, Some("edit".to_string()));
            }
            _ => panic!("Expected AssetDetails variant"),
        }
    }

    #[test]
    fn test_from_str_unknown_path() {
        let result = AppRoutes::from_str("/unknown/path");
        assert!(result.is_err());
    }

    #[test]
    fn test_from_asref_str_with_fallback() {
        let route: AppRoutes = "/unknown/path".into();
        assert_eq!(route, AppRoutes::NotFound);
    }

    #[test]
    fn test_from_asref_str_valid_route() {
        let route: AppRoutes = "/profile".into();
        assert_eq!(route, AppRoutes::Profile);
    }
}
