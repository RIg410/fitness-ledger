use std::sync::Arc;
use model::user::UserPreCell;
use mongodb::{Collection, Database};
use eyre::Result;

const PRE_CELL_COLLECTION: &str = "users_precell";

#[derive(Clone)]
pub struct PreCellStore {
    pub(crate) pre_cell: Arc<Collection<UserPreCell>>,
}

impl PreCellStore {
    pub(crate) async fn new(db: &Database) -> Result<Self> {
        Ok(PreCellStore {
            pre_cell: Arc::new(db.collection(PRE_CELL_COLLECTION)),
        })
    }

    
}
