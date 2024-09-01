use std::sync::Arc;

use calendar::Calendar;
use storage::training::TrainingStore;
use storage::{user::UserStore, Storage};

pub mod calendar;
pub mod training;
mod users;
pub use users::*;

const MAX_WEEKS: i64 = 12;

#[derive(Clone)]
pub struct Ledger {
    pub(crate) users: UserStore,
    pub calendar: Arc<Calendar>,
    pub(crate) training: TrainingStore,
}

impl Ledger {
    pub fn new(storage: Storage) -> Self {
        Ledger {
            users: storage.users,
            calendar: Arc::new(Calendar::new(storage.schedule)),
            training: storage.training,
        }
    }
}
