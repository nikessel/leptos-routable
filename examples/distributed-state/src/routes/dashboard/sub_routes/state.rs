use reactive_stores::Store;

#[derive(Store, Default, Debug)]
pub struct State {
    pub settings: super::settings::state::State,
}
