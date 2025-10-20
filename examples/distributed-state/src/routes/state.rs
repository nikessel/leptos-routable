use reactive_stores::Store;

#[derive(Store, Default, Debug)]
pub struct State {
    pub counter: u32,
    pub index: super::index::state::State,
    pub dashboard: super::dashboard::state::State,
    pub not_found: super::not_found::state::State,
}
