use leptos::prelude::*;
use leptos_meta::{provide_meta_context, Meta, Title};
use leptos_routable::prelude::*;
use leptos_router::components::{Router, A};
use reactive_stores::{Store, Field};

// ============================================================================
// State definitions - Must match route variant names (lowercase)
// ============================================================================

#[derive(Store, Default, Debug)]
pub struct AppRoutesState {
    home: HomeState,
    about: AboutState,
    dashboard: DashboardState,
    not_found: NotFoundState,
}

#[derive(Store, Default, Debug)]
pub struct HomeState {
    pub counter: i32,
    pub message: String,
}

#[derive(Store, Default, Debug)]
pub struct AboutState {
    pub visits: u32,
}

#[derive(Store, Default, Debug)]
pub struct DashboardState {
    pub user: Option<User>,
    pub notifications: Vec<String>,
    // Nested states for nested routes (snake_case naming)
    pub dashboard_analytics: AnalyticsState,
    pub dashboard_settings: SettingsState,
}

#[derive(Store, Default, Debug)]
pub struct AnalyticsState {
    pub page_views: u64,
}

#[derive(Store, Default, Debug)]
pub struct SettingsState {
    pub theme: String,
}

#[derive(Store, Default, Debug)]
pub struct NotFoundState {
    pub attempted_path: String,
}

#[derive(Clone, Debug)]
pub struct User {
    pub name: String,
    pub id: u64,
}

// ============================================================================
// Route definitions with state support
// ============================================================================

#[derive(Routable, PartialEq, Debug, Clone)]
#[routes(
    view_prefix = "",
    view_suffix = "View",
    state_suffix = "State",  // Enables state support
    transition = false
)]
pub enum AppRoutes {
    #[route(path = "/")]
    Home,

    #[route(path = "/about")]
    About,

    #[parent_route(path = "/dashboard")]
    Dashboard(DashboardRoutes),

    #[fallback]
    #[route(path = "/404")]
    NotFound,
}

#[derive(Routable, PartialEq, Debug, Clone)]
#[routes(view_prefix = "", view_suffix = "View", transition = false)]
pub enum DashboardRoutes {
    #[route(path = "/analytics")]
    DashboardAnalytics,

    #[route(path = "/settings")]
    DashboardSettings,
}

// ============================================================================
// View components that use the state
// ============================================================================

#[component]
pub fn HomeView() -> impl IntoView {
    // Access the state provided by the router
    let state = use_context::<Field<HomeState>>()
        .expect("HomeState should be provided");

    view! {
        <div class="p-4">
            <h1 class="text-2xl font-bold">"Home"</h1>
            <p>"Counter: " {move || state.counter().get()}</p>
            <p>"Message: " {move || state.message().get()}</p>
            <button
                class="px-4 py-2 bg-blue-500 text-white rounded"
                on:click=move |_| {
                    state.counter().update(|c| *c += 1);
                    state.message().set(format!("Clicked {} times", state.counter().get() + 1));
                }
            >
                "Increment"
            </button>
        </div>
    }
}

#[component]
pub fn AboutView() -> impl IntoView {
    let state = use_context::<Field<AboutState>>()
        .expect("AboutState should be provided");

    // Increment visits on mount
    Effect::new(move |_| {
        state.visits().update(|v| *v += 1);
    });

    view! {
        <div class="p-4">
            <h1 class="text-2xl font-bold">"About"</h1>
            <p>"This page has been visited " {move || state.visits().get()} " times"</p>
        </div>
    }
}

#[component]
pub fn DashboardView() -> impl IntoView {
    // Parent route state is available
    let state = use_context::<Field<DashboardState>>()
        .expect("DashboardState should be provided");

    view! {
        <div class="p-4">
            <h1 class="text-2xl font-bold">"Dashboard"</h1>
            <div class="mb-4">
                {move || match state.user().get() {
                    Some(user) => view! { <p>"Welcome, " {user.name} "!"</p> }.into_any(),
                    None => view! {
                        <div>
                            <p>"Not logged in"</p>
                            <button
                                class="px-4 py-2 bg-green-500 text-white rounded"
                                on:click=move |_| {
                                    state.user().set(Some(User {
                                        name: "Alice".to_string(),
                                        id: 42,
                                    }));
                                }
                            >
                                "Login as Alice"
                            </button>
                        </div>
                    }.into_any()
                }}
            </div>
            <div class="border-t pt-4">
                <p class="font-semibold">"Notifications:"</p>
                {move || {
                    let notifications = state.notifications().get();
                    if notifications.is_empty() {
                        view! { <p class="text-gray-500">"No notifications"</p> }.into_any()
                    } else {
                        view! {
                            <ul>
                                {notifications.into_iter()
                                    .map(|n| view! { <li>{n}</li> })
                                    .collect_view()}
                            </ul>
                        }.into_any()
                    }
                }}
                <button
                    class="mt-2 px-4 py-2 bg-blue-500 text-white rounded"
                    on:click=move |_| {
                        state.notifications().update(|n| {
                            n.push(format!("Notification {}", n.len() + 1));
                        });
                    }
                >
                    "Add Notification"
                </button>
            </div>
        </div>
    }
}

#[component]
pub fn DashboardAnalyticsView() -> impl IntoView {
    // Can access parent state
    let parent_state = use_context::<Field<DashboardState>>()
        .expect("DashboardState from parent");

    view! {
        <div class="p-4">
            <h2 class="text-xl font-bold">"Analytics"</h2>
            {move || match parent_state.user().get() {
                Some(user) => view! {
                    <p>"Showing analytics for " {user.name}</p>
                }.into_any(),
                None => view! {
                    <p class="text-red-500">"Login required to view analytics"</p>
                }.into_any()
            }}
            <p>"Page views: " {move || parent_state.dashboard_analytics().page_views().get()}</p>
            <button
                class="mt-2 px-4 py-2 bg-purple-500 text-white rounded"
                on:click=move |_| {
                    parent_state.dashboard_analytics().page_views().update(|v| *v += 1);
                }
            >
                "Simulate Page View"
            </button>
        </div>
    }
}

#[component]
pub fn DashboardSettingsView() -> impl IntoView {
    let parent_state = use_context::<Field<DashboardState>>()
        .expect("DashboardState from parent");

    view! {
        <div class="p-4">
            <h2 class="text-xl font-bold">"Settings"</h2>
            <p>"Current theme: " {move || parent_state.dashboard_settings().theme().get()}</p>
            <div class="mt-4 space-x-2">
                <button
                    class="px-4 py-2 bg-gray-700 text-white rounded"
                    on:click=move |_| parent_state.dashboard_settings().theme().set("dark".to_string())
                >
                    "Dark Theme"
                </button>
                <button
                    class="px-4 py-2 bg-gray-200 text-black rounded"
                    on:click=move |_| parent_state.dashboard_settings().theme().set("light".to_string())
                >
                    "Light Theme"
                </button>
            </div>
            {move || {
                let notification_count = parent_state.notifications().get().len();
                view! {
                    <p class="mt-4 text-sm text-gray-600">
                        "You have " {notification_count} " notifications (from parent state)"
                    </p>
                }
            }}
        </div>
    }
}

#[component]
pub fn NotFoundView() -> impl IntoView {
    let state = use_context::<Field<NotFoundState>>()
        .expect("NotFoundState should be provided");

    view! {
        <div class="p-4">
            <h1 class="text-2xl font-bold text-red-600">"404 - Not Found"</h1>
            <p>"The page you're looking for doesn't exist."</p>
            {move || {
                let path = state.attempted_path().get();
                if !path.is_empty() {
                    view! { <p class="text-sm text-gray-600">"Attempted path: " {path}</p> }.into_any()
                } else {
                    view! { <span /> }.into_any()
                }
            }}
        </div>
    }
}

// ============================================================================
// App component
// ============================================================================

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        <Meta charset="UTF-8"/>
        <Meta name="viewport" content="width=device-width, initial-scale=1.0"/>
        <Title text="State Support Example"/>

        <Router>
            <nav class="bg-gray-800 text-white p-4">
                <div class="container mx-auto flex space-x-4">
                    <A href=AppRoutes::Home attr:class="hover:text-gray-300">"Home"</A>
                    <A href=AppRoutes::About attr:class="hover:text-gray-300">"About"</A>
                    <A href=AppRoutes::Dashboard(DashboardRoutes::DashboardAnalytics) attr:class="hover:text-gray-300">
                        "Dashboard"
                    </A>
                    <A href=AppRoutes::Dashboard(DashboardRoutes::DashboardSettings) attr:class="hover:text-gray-300">
                        "Settings"
                    </A>
                </div>
            </nav>
            <main class="container mx-auto">
                {move || AppRoutes::routes()}
            </main>
        </Router>
    }
}