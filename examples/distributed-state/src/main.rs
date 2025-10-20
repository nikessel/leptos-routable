use leptos::prelude::*;
use distributed_state_example::AppRoutes;
use leptos_router::components::Router;
use leptos_routable::Routable;

fn main() {
    console_error_panic_hook::set_once();
    leptos::mount::mount_to_body(|| view! { <App /> });
}

#[component]
fn App() -> impl IntoView {
    view! {
        <div>
            <h1>"Distributed State Test"</h1>
            <Router>
                {move || AppRoutes::routes()}
            </Router>
        </div>
    }
}
