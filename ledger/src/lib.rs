use storage::schedule::ScheduleStore;
use storage::{user::UserStore, Storage};
mod schedule;
mod users;
pub use users::*;

const MAX_WEEKS: i64 = 12;

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
