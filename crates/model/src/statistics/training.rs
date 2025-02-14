use crate::{program::TrainingType, training::Training};
use bson::oid::ObjectId;
use chrono::{Datelike as _, Timelike, Weekday};
use std::collections::HashMap;
use std::fmt;
use std::fmt::Display;
use std::fmt::Formatter;

#[derive(Debug, Default)]
pub struct TrainingsStat {
    pub by_program: HashMap<ObjectId, TrainingStat>,
    pub by_instructor: HashMap<ObjectId, TrainingStat>,
    pub by_room: HashMap<ObjectId, TrainingStat>,
    pub by_type: HashMap<StatTrainingType, TrainingStat>,
    pub by_weekday: HashMap<Weekday, TrainingStat>,
    pub by_time: HashMap<u32, TrainingStat>,
    pub programs: HashMap<ObjectId, String>,
}

impl TrainingsStat {
    pub fn extend(&mut self, training: &Training) {
        let slot = training.get_slot();
        self.by_program
            .entry(training.proto_id)
            .or_default()
            .extend(training);
        self.by_instructor
            .entry(training.instructor)
            .or_default()
            .extend(training);
        self.by_room
            .entry(slot.room())
            .or_default()
            .extend(training);
        self.by_type
            .entry(training.tp.into())
            .or_default()
            .extend(training);
        self.by_weekday
            .entry(slot.day_id().local().weekday())
            .or_default()
            .extend(training);
        let time = slot.start_at();
        self.by_time
            .entry(time.hour())
            .or_default()
            .extend(training);

        self.programs
            .entry(training.proto_id)
            .or_insert_with(|| training.name.clone());
    }
}

#[derive(Debug, Default)]
pub struct TrainingStat {
    pub trainings_count: u64,
    pub total_clients: u64,
    pub total_earned: i64,
    pub trainings_with_out_clients: u64,
    pub canceled_trainings: u64,
}

impl TrainingStat {
    pub fn extend(&mut self, other: &Training) {
        self.trainings_count += 1;
        self.total_clients += other.clients.len() as u64;
        if let Some(stat) = &other.statistics {
            self.total_earned += stat.earned.int_part();
        }
        if other.clients.is_empty() {
            self.trainings_with_out_clients += 1;
        }
        if other.is_canceled {
            self.canceled_trainings += 1;
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StatTrainingType {
    Group,
    Personal,
    SubRent,
}

impl Display for StatTrainingType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl From<TrainingType> for StatTrainingType {
    fn from(tp: TrainingType) -> Self {
        match tp {
            TrainingType::Group { .. } => StatTrainingType::Group,
            TrainingType::Personal { .. } => StatTrainingType::Personal,
            TrainingType::SubRent { .. } => StatTrainingType::SubRent,
        }
    }
}
