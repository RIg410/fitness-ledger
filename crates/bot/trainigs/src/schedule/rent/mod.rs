use chrono::{DateTime, Duration, Local};
use mongodb::bson::oid::ObjectId;

#[derive(Default, Clone)]
pub struct RentPreset {
    pub day: Option<DateTime<Local>>,
    pub date_time: Option<DateTime<Local>>,
    pub room: Option<ObjectId>,
    pub duration: Option<Duration>,
}