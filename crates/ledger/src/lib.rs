use calendar::Calendar;
use storage::session::Db;
use storage::Storage;

pub mod calendar;
pub mod training;
mod users;
use training::Programs;
pub use users::*;

#[derive(Clone)]
pub struct Ledger {
    pub db: Db,
    pub users: Users,
    pub calendar: Calendar,
    pub programs: Programs,
}

impl Ledger {
    pub fn new(storage: Storage) -> Self {
        let programs = Programs::new(storage.training);
        let calendar = Calendar::new(storage.calendar, storage.users.clone(), programs.clone());
        let users = Users::new(storage.users, calendar.clone());
        Ledger {
            users,
            calendar,
            programs,
            db: storage.db,
        }
    }
}
