use reactive_stores::Store;

#[derive(Store, Default, Debug)]
pub struct State {
    pub toggle: bool,
    pub sub_state: super::sub_routes::state::State,
}
