use chrono::{DateTime, Local};

pub struct DayStat {
    pub dt: DateTime<Local>,
    pub trainings: Vec<TrainingStat>,
}

pub struct TrainingStat {
    pub name: String,
    pub start_at: DateTime<Local>,
    pub clients: u32,
    pub instructor: Option<String>,
    pub earned: i64,
    pub paid: i64,
    pub tp: TrainingType,
    pub room: String,
}


pub enum TrainingType {
    Group,
    Personal,
    Rent,
}