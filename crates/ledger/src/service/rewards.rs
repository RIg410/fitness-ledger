use std::ops::Deref;
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

impl Deref for Rewards {
    type Target = RewardsStore;

    fn deref(&self) -> &Self::Target {
        &self.store
    }
}
