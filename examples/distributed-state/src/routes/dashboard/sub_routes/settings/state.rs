use reactive_stores::Store;

#[derive(Store, Default, Debug)]
pub struct State {
    pub value: String,
}
