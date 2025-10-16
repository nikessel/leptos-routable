# State Support Guide for leptos-routable

The state support feature automatically provides reactive state in context for routes and their children.

## Key Concepts

### 1. Enable State Support (Root Router Only)

Add `state_suffix = "State"` to your **root route enum only**:

```rust
#[derive(Routable, PartialEq, Debug, Clone)]
#[routes(
    view_prefix = "",
    view_suffix = "View",
    state_suffix = "State",  // ← Only needed on root router!
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

// Nested router - NO state_suffix needed
#[derive(Routable, PartialEq, Debug, Clone)]
#[routes(view_prefix = "", view_suffix = "View", transition = false)]
pub enum DashboardRoutes {
    #[route(path = "/analytics")]
    DashboardAnalytics,

    #[route(path = "/settings")]
    DashboardSettings,
}
```

### 2. State Structure Requirements

**Every route must have a corresponding state struct** (can be empty):

```rust
use reactive_stores::Store;

// Root state - name is {RootEnum}{state_suffix}
#[derive(Store, Default, Debug)]
pub struct AppRoutesState {
    pub home: HomeState,
    pub about: AboutState,
    pub dashboard: DashboardState,
    pub not_found: NotFoundState,
}

// Individual state structs - can be empty
#[derive(Store, Default, Debug)]
pub struct HomeState {
    pub counter: i32,  // Optional fields
}

#[derive(Store, Default, Debug)]
pub struct AboutState {} // Can be empty!
```

### 3. Parent Routes Need sub_state

Routes with nested routes **must** have a `sub_state` field:

```rust
#[derive(Store, Default, Debug)]
pub struct DashboardState {
    // Parent route's own fields (optional)
    pub user: Option<User>,
    pub notifications: Vec<String>,

    // Required: sub_state field of type {ParentName}SubState
    pub sub_state: DashboardSubState,
}

// SubState struct contains fields for nested routes
#[derive(Store, Default, Debug)]
pub struct DashboardSubState {
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
```

### 4. Field Naming Convention

Fields must use **snake_case** version of route names:

| Route Variant | State Field Name |
|--------------|------------------|
| `Home` | `home` |
| `About` | `about` |
| `NotFound` | `not_found` |
| `Dashboard` | `dashboard` |
| `DashboardAnalytics` | `dashboard_analytics` |

### 5. Auto-Generated Context Helpers

The macro generates convenient helper methods:

```rust
impl YourState {
    // Returns Option<Store<T>> for root, Option<Field<T>> for others
    pub fn use_context() -> Option<...> { ... }

    // Returns context or panics with helpful message
    pub fn expect_context() -> ... { ... }
}
```

### 6. Using State in Views

```rust
#[component]
pub fn HomeView() -> impl IntoView {
    // Simply use the generated helper
    let state = HomeState::expect_context();

    view! {
        <div>
            <p>"Counter: " {move || state.counter().get()}</p>
            <button on:click=move |_| {
                state.counter().update(|c| *c += 1);
            }>
                "Increment"
            </button>
        </div>
    }
}

#[component]
pub fn DashboardAnalyticsView() -> impl IntoView {
    // Access parent state
    let parent_state = DashboardState::expect_context();

    // Access sub_state for nested routes
    let sub_state = DashboardSubState::expect_context();

    view! {
        <div>
            // Use parent state
            {move || match parent_state.user().get() {
                Some(user) => view! { <p>"User: " {user.name}</p> },
                None => view! { <p>"Not logged in"</p> }
            }}

            // Use nested route state
            <p>"Views: " {move || sub_state.dashboard_analytics().page_views().get()}</p>
        </div>
    }
}
```

## Important Notes on reactive_stores

### When to use #[store] attribute

Most fields work automatically, but you need `#[store]` for:

1. **Keyed collections**:
   ```rust
   #[store(key: usize = |todo| todo.id)]
   todos: Vec<Todo>,
   ```

2. **Complex recursive types**:
   ```rust
   #[store]
   child: Option<Box<Self>>,
   ```

**You DON'T need #[store] for**:
- Primitive types (`i32`, `String`, `bool`)
- Nested Store types (like `sub_state: DashboardSubState`)
- Simple `Option<T>` or `Vec<T>` (unless you want keyed access)

### Reactivity Rules

1. All fields in a Store-derived struct are reactive
2. Updates propagate to parents and children, not siblings
3. The macro handles all the reactive plumbing automatically

## Complete Example

See `/workspace/examples/state-support/src/lib.rs` for a full working implementation.

## Quick Checklist

- [ ] Add `state_suffix` to root router only
- [ ] Create state struct named `{RootEnum}{state_suffix}`
- [ ] Add field for each route (snake_case naming)
- [ ] For parent routes: add `sub_state: {ParentName}SubState`
- [ ] Create SubState structs with fields for nested routes
- [ ] All state structs derive `Store` and `Default`
- [ ] Use `State::expect_context()` in views

## Troubleshooting

**Compilation Error: "field doesn't exist"**
- Check field names are snake_case version of route names
- Ensure parent routes have `sub_state` field
- Verify SubState type is named `{ParentName}SubState`

**Runtime Panic: "should be provided"**
- Make sure state_suffix is set on root router
- Check that all required state structs exist
- Verify state struct naming follows pattern

**State Not Reactive**
- Ensure all state structs derive `Store`
- Use `.get()`, `.set()`, `.update()` for reactive access
- Check you're not cloning non-reactive values