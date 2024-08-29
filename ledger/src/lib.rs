use storage::schedule::ScheduleStore;
use storage::{user::UserStore, Storage};
mod users;
mod schedule;
pub use users::*;
pub use schedule::*;

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
