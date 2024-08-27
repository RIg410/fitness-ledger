use std::sync::Arc;

use storage::Storage;

#[derive(Clone)]
pub struct Ledger {
    storage: Arc<Storage>,
}

impl Ledger {
    pub fn new(storage: Storage) -> Self {
        Ledger { storage: Arc::new(storage) }
    }
}
