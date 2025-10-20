use leptos::prelude::*;
use crate::routes::dashboard::sub_routes::settings::state::StateStoreFields;

#[component]
pub fn View() -> impl IntoView {
    let state = crate::routes::dashboard::sub_routes::settings::state::State::expect_context();

    view! {
        <div>
            <h3>"Settings"</h3>
            <p>"Value: " {move || state.value().get()}</p>
        </div>
    }
}
