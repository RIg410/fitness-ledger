use crate::{ids::DayId, slot::Slot, training::Training};
use chrono::{DateTime, Datelike, Local, Utc};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Day {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    #[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    date_time: DateTime<Utc>,
    pub weekday: chrono::Weekday,
    pub training: Vec<Training>,
    #[serde(default)]
    pub version: u64,
}

impl Day {
    pub fn new(day: DayId) -> Day {
        Day {
            weekday: day.local().weekday(),
            training: Vec::new(),
            id: ObjectId::new(),
            date_time: day.id(),
            version: 0,
        }
    }

    pub fn check_collision(&self, new: &Training) -> Option<Collision> {
        let new_slot = new.get_slot();
        for old in &self.training {
            if old.is_canceled {
                continue;
            }

            if old.get_slot().has_conflict(&new_slot) {
                return Some(Collision {
                    day_id: self.day_id(),
                    training_id: old.id,
                });
            }
        }

        None
    }

    pub fn copy_day(id: DayId, day: Day) -> Day {
        let training = day
            .training
            .into_iter()
            .filter(|t| !t.is_one_time)
            .map(|t| Training::with_day_and_training(id, t))
            .collect::<Vec<_>>();

        Day {
            id: ObjectId::new(),
            date_time: id.id(),
            weekday: id.week_day(),
            training,
            version: 0,
        }
    }

    pub fn day_id(&self) -> DayId {
        unsafe { DayId::from_utc(self.date_time) }
    }

    pub fn day_date(&self) -> DateTime<Local> {
        self.date_time.with_timezone(&Local)
    }

    pub fn has_conflict(&self) -> bool {
        let mut slots: Vec<Slot> = Vec::new();
        for training in &self.training {
            if training.is_canceled {
                continue;
            }

            let slot = training.get_slot();
            if slots.iter().any(|s| s.has_conflict(&slot)) {
                return true;
            }

            slots.push(slot);
        }

        false
    }
}

pub struct Collision {
    pub day_id: DayId,
    pub training_id: ObjectId,
}
