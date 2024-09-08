use storage::treasury::TreasuryStore;

#[derive(Clone)]
pub struct Treasury {
    store: TreasuryStore,
}

impl Treasury {
    pub fn new(store: TreasuryStore) -> Self {
        Treasury { store }
    }
}
