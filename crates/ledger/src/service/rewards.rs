use std::{ops::Deref, sync::Arc};
use storage::rewards::RewardsStore;

pub struct Rewards {
    store: Arc<RewardsStore>,
}

impl Rewards {
    pub(crate) fn new(store: Arc<RewardsStore>) -> Self {
        Rewards { store }
    }
}

impl Deref for Rewards {
    type Target = RewardsStore;

    fn deref(&self) -> &Self::Target {
        &self.store
    }
}
