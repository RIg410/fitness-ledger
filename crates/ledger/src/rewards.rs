use storage::rewards::RewardsStore;

#[derive(Clone)]
pub struct Rewards {
    store: RewardsStore,
}

impl Rewards {
    pub(crate) fn new(store: RewardsStore) -> Self {
        Rewards { store }
    }
}
