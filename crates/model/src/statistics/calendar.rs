use crate::{day::Day, decimal::Decimal, training::Training};
use bson::oid::ObjectId;
use chrono::{DateTime, Datelike as _, Local, Timelike, Weekday};
use core::fmt;
use std::{collections::HashMap, fmt::Display};

#[derive(Default)]
pub struct LedgerStatistics {
    pub by_program: HashMap<ObjectId, EntryInfo>,
    pub by_weekday: HashMap<Weekday, EntryInfo>,
    pub by_instructor: HashMap<ObjectId, EntryInfo>,
    pub by_time_slot: HashMap<TimeSlot, EntryInfo>,
    pub users: HashMap<ObjectId, UserStat>,
}

impl LedgerStatistics {
    pub fn extend(&mut self, day: Day) {
        let weekday = day.day_date().weekday();
        for training in day.training {
            if !training.is_processed || training.is_canceled {
                continue;
            }
            let start_at = training.get_slot().start_at();

            let info = EntryInfo::new(&training);
            self.by_program
                .entry(training.proto_id)
                .or_default()
                .extend(&info);
            self.by_weekday.entry(weekday).or_default().extend(&info);
            self.by_instructor
                .entry(training.instructor)
                .or_default()
                .extend(&info);
            self.by_time_slot
                .entry(TimeSlot::new(start_at))
                .or_default()
                .extend(&info);

            for client in training.clients {
                let user_stat = self.users.entry(client).or_default();
                user_stat.total += 1;
                *user_stat.programs.entry(training.proto_id).or_default() += 1;
                *user_stat.weekdays.entry(weekday).or_default() += 1;
                *user_stat
                    .time_slots
                    .entry(TimeSlot::new(start_at))
                    .or_default() += 1;
                *user_stat
                    .instructors
                    .entry(training.instructor)
                    .or_default() += 1;
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Copy)]
pub struct TimeSlot {
    hour: u8,
}

impl Display for TimeSlot {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:02}:00", self.hour)
    }
}

impl TimeSlot {
    pub fn new(date_time: DateTime<Local>) -> Self {
        Self {
            hour: date_time.hour() as u8,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct EntryInfo {
    pub total_training: u32,
    pub earn: Decimal,
    pub reward: Decimal,
    pub visit: u32,
    pub without_clients: u32,
}

impl EntryInfo {
    pub fn new(training: &Training) -> Self {
        if let Some(stat) = training.statistics.as_ref() {
            Self {
                total_training: 1,
                earn: stat.earned,
                reward: stat.couch_rewards,
                visit: training.clients.len() as u32,
                without_clients: if training.clients.is_empty() { 1 } else { 0 },
            }
        } else {
            Self {
                total_training: 1,
                earn: Decimal::zero(),
                reward: Decimal::zero(),
                visit: training.clients.len() as u32,
                without_clients: if training.clients.is_empty() { 1 } else { 0 },
            }
        }
    }

    pub fn avg_visits(&self) -> f64 {
        if self.visit == 0 {
            0.0
        } else {
            self.visit as f64 / self.total_training as f64
        }
    }

    pub fn extend(&mut self, info: &EntryInfo) {
        self.total_training += info.total_training;
        self.earn += info.earn;
        self.reward += info.reward;
        self.visit += info.visit;
        self.without_clients += info.without_clients;
    }
}

#[derive(Clone, Debug, Default)]
pub struct UserStat {
    pub total: u32,
    pub programs: HashMap<ObjectId, u32>,
    pub weekdays: HashMap<Weekday, u32>,
    pub time_slots: HashMap<TimeSlot, u32>,
    pub instructors: HashMap<ObjectId, u32>,
}
