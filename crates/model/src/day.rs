use crate::{ids::DayId, training::Training};
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

    pub fn day_id(&self) -> DayId {
        unsafe { DayId::from_utc(self.date_time) }
    }

    pub fn add_training(&mut self, training: Training) -> bool {
        let new_training_start_at = training.start_at_local();
        let new_training_end_at = training.start_at_local()
            + chrono::Duration::minutes(training.duration_min as i64)
            + chrono::Duration::seconds(1);
        let conflict = self
            .training
            .iter()
            .map(|t| {
                (
                    t.start_at_local(),
                    t.start_at_local() + chrono::Duration::minutes(t.duration_min as i64),
                )
            })
            .any(|(start, end)| {
                (new_training_start_at >= start && new_training_start_at < end)
                    || (new_training_end_at > start && new_training_end_at <= end)
            });
        if !conflict {
            self.training.push(training);
            self.training.sort_by(|a, b| a.start_at.cmp(&b.start_at));
            true
        } else {
            false
        }
    }

    pub fn remove_training(&mut self, training_id: ObjectId) -> bool {
        let index = self.training.iter().position(|t| t.id == training_id);
        if let Some(index) = index {
            self.training.remove(index);
            true
        } else {
            false
        }
    }

    pub fn day_date(&self) -> DateTime<Local> {
        self.date_time.with_timezone(&Local)
    }

    pub fn copy(self, id: DayId) -> Day {
        let training = self
            .training
            .into_iter()
            .map(|t| t.change_date(id))
            .collect::<Vec<_>>();

        Day {
            id: ObjectId::new(),
            date_time: id.id(),
            weekday: id.week_day(),
            training,
            version: 0,
        }
    }
}
