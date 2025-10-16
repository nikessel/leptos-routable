# How to Use State Support in leptos-routable

The state support feature automatically provides reactive state in context for your routes. Here's a complete walkthrough:

## Step 1: Enable State Support

Add `state_suffix = "State"` to your route enum:

```rust
#[derive(Routable, PartialEq, Debug, Clone)]
#[routes(
    view_prefix = "",
    view_suffix = "View",
    state_suffix = "State",  // ← Enable state support
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
```

## Step 2: Create State Structs

Create a state struct named `{EnumName}{state_suffix}` with fields for nested routes:

```rust
use reactive_stores::{Store, Field};

#[derive(Store, Default, Debug)]
pub struct AppRoutesState {
    // You can have any fields you want
    pub shared_data: String,
    pub theme: String,

    // BUT: Parent routes (those with nested routes) MUST have a field
    // Field name = snake_case of variant name
    dashboard: DashboardState,  // Required for Dashboard(DashboardRoutes)
}
```

**Key Rules:**
- ✅ Parent routes MUST have a corresponding field (snake_case naming)
- ✅ Non-parent routes don't need fields (but can have them if you want)
- ✅ You can add any other fields you need
- ✅ All state structs must derive `Store` and `Default`

## Step 3: Handle Nested Routes

For parent routes with children, include fields for the nested routes:

```rust
#[derive(Store, Default, Debug)]
pub struct DashboardState {
    // Parent route can have its own state
    pub user: Option<User>,
    pub notifications: Vec<String>,

    // Fields for nested routes (snake_case!)
    dashboard_analytics: AnalyticsState,  // For DashboardAnalytics
    dashboard_settings: SettingsState,    // For DashboardSettings
}

#[derive(Store, Default, Debug)]
pub struct AnalyticsState {
    pub page_views: u64,
}

#[derive(Store, Default, Debug)]
pub struct SettingsState {
    pub theme: String,
}
```

## Step 4: Access State in Views

Parent routes get their state field:

```rust
#[component]
pub fn DashboardView() -> impl IntoView {
    // Parent route gets its state field
    let state = use_context::<Field<DashboardState>>()
        .expect("DashboardState should be provided");

    view! {
        <div>
            <h1>"Dashboard"</h1>
            // Access parent state
            {move || match state.user().get() {
                Some(user) => view! { <p>"Welcome " {user.name}</p> },
                None => view! { <p>"Not logged in"</p> }
            }}
        </div>
    }
}
```

Nested routes can access parent state AND their own nested state:

```rust
#[component]
pub fn DashboardAnalyticsView() -> impl IntoView {
    // Nested route gets parent state
    let parent_state = use_context::<Field<DashboardState>>()
        .expect("DashboardState from parent");

    view! {
        <div>
            // Access parent's user field
            {move || match parent_state.user().get() {
                Some(user) => view! { <p>"Analytics for " {user.name}</p> },
                None => view! { <p>"Login required"</p> }
            }}

            // Access nested state - maintains full reactivity!
            <p>"Views: " {move || parent_state.dashboard_analytics().page_views().get()}</p>

            // Updates propagate reactively
            <button on:click=move |_| {
                parent_state.dashboard_analytics().page_views().update(|v| *v += 1);
            }>
                "Increment"
            </button>
        </div>
    }
}
```

## Field Naming Convention

The macro converts CamelCase route names to snake_case field names:

| Route Variant | State Field Name |
|--------------|-----------------|
| `Home` | `home` |
| `About` | `about` |
| `NotFound` | `not_found` |
| `Dashboard` | `dashboard` |
| `DashboardAnalytics` | `dashboard_analytics` |
| `UserProfile` | `user_profile` |

## Important Notes

1. **Only parent routes are enforced**: The macro only validates that parent routes (those with nested routes) have corresponding state fields. Other routes can optionally have state.

2. **Add any fields you want**: Your state structs can have additional fields beyond the required ones for nested routes.

3. **Reactivity is maintained**: Using `Field<T>` ensures all updates are reactive through the entire chain.

4. **Context type matters**: Views receive `Field<StateType>`, not `Store<StateType>`.

## Complete Working Example

See `/workspace/examples/state-support/src/lib.rs` for a full implementation showing:
- Parent routes with state
- Nested routes accessing parent state
- Reactive updates propagating through the tree
- Additional custom fields in state structs