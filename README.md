# Leptos Routable Example

This repository demonstrates a simple **nested routing** setup in [Leptos](https://github.com/leptos-rs/leptos),
allowing you to group pages under a shared path while still defining each route in a clear, type-safe way.

## Quick Peek: Nested Routes

```rust
use leptos::prelude::*;
use leptos_router::{Router, components::A};
use leptos_routable::prelude::*;
use my_components::*;

// Top-level routes
#[derive(Routable)]
pub enum AppRoutes {
    #[route(path = "/")]
    Home,
    #[parent_route(path = "/dashboard")]
    Dashboard(DashboardRoutes),
    #[fallback]
    #[route(path = "/404")]
    NotFound,
}

// Child routes under `/dashboard`
#[derive(Routable)]
pub enum DashboardRoutes {
    #[route(path = "")]
    DashboardHome,
    #[route(path = "/settings")]
    DashboardSettings,
    #[fallback]
    DashboardNotFound,
}

// Render the main app with a router
#[component]
pub fn App() -> impl IntoView {
    view! {
        <Router>
            // Simple Nav
            <nav>
                <A href=AppRoutes::Home>"Home"</A>
                <A href=AppRoutes::Dashboard(DashboardRoutes::DashboardHome)>"Dashboard"</A>
            </nav>

            // Route content goes here
            {move || AppRoutes::routes()}
        </Router>
    }
}
```

- **`AppRoutes::Dashboard(DashboardRoutes::DashboardHome)`**  
  Leptos generates a properly nested URL—`/dashboard`—with no extra string typing.
- **`DashboardRoutes::DashboardSettings`**  
  Expands the route to `/dashboard/settings`.

## Highlighted Features

- **Easy Route Definitions**  
  `#[derive(Routable)]` handles URL generation and route matching in one neat enum.
- **Nested Paths**  
  Group your routes under parent routes (e.g., `/dashboard`) and define child routes for a cleaner structure.
- **Zero-String Linking**  
  Use `<A href=AppRoutes::SomeRoute>` to navigate without manually typing paths—less chance of typos!

## Contributing

Pull requests and suggestions are welcome. For more info on Leptos and building frontends in Rust, check
out [Leptos’ official repository](https://github.com/leptos-rs/leptos) or join the community Discord.
