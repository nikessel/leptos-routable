use leptos::prelude::*;
use leptos_router::components::Outlet;

#[component]
pub fn Layout() -> impl IntoView {
    view! {
        <div>
            <h2>"Dashboard"</h2>
            <Outlet />
        </div>
    }
}
