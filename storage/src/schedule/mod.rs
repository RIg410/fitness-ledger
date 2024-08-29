pub mod model;

use std::sync::Arc;
use mongodb::{Collection, Database};

const COLLECTION: &str = "schedule";

#[derive(Clone)]
pub struct ScheduleStore {
    pub(crate) schedule: Arc<Collection<()>>,
}

impl ScheduleStore {
    pub(crate) fn new(db: &Database) -> Self {
        let schedule = db.collection(COLLECTION);
        ScheduleStore {
            schedule: Arc::new(schedule),
        }
    }
}