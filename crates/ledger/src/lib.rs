use calendar::Calendar;
use storage::session::Db;
use storage::training::TrainingStore;
use storage::Storage;

pub mod calendar;
pub mod training;
mod users;
pub use users::*;

#[derive(Clone)]
pub struct Ledger {
    pub db: Db,
    pub users: Users,
    pub calendar: Calendar,
    pub(crate) training: TrainingStore,
}

impl Ledger {
    pub fn new(storage: Storage) -> Self {
        let calendar = Calendar::new(storage.calendar);
        let users = Users::new(storage.users, calendar.clone());
        Ledger {
            users,
            calendar,
            training: storage.training,
            db: storage.db,
        }
    }
}
