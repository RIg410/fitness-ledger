use eyre::eyre;
use eyre::Result;
use log::{info, warn};
use storage::schedule::ScheduleStore;
use storage::{user::UserStore, Storage};
mod users;
pub use users::*;

#[derive(Clone)]
pub struct Ledger {
    pub(crate) storage: UserStore,
    pub(crate) schedule: ScheduleStore,
}

impl Ledger {
    pub fn new(storage: Storage) -> Self {
        Ledger {
            storage: storage.users,
            schedule: storage.schedule,
        }
    }
}
