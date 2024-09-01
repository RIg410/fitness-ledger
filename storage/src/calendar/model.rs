use crate::training::model::Training;
use chrono::{DateTime, Local};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Week {
    pub id: DateTime<Local>,
    pub days: [Day; 7],
}

impl Week {
    pub fn days(&self) -> impl Iterator<Item = (&Day, DateTime<Local>)> {
        self.days
            .iter()
            .enumerate()
            .map(move |(i, day)| (day, self.id + chrono::Duration::days(i as i64)))
    }

    pub fn new(first_date: DateTime<Local>) -> Week {
        Week {
            id: first_date,
            days: [
                Day::new(chrono::Weekday::Mon),
                Day::new(chrono::Weekday::Tue),
                Day::new(chrono::Weekday::Wed),
                Day::new(chrono::Weekday::Thu),
                Day::new(chrono::Weekday::Fri),
                Day::new(chrono::Weekday::Sat),
                Day::new(chrono::Weekday::Sun),
            ],
        }
    }

    pub fn get_day_mut(&mut self, weekday: chrono::Weekday) -> &mut Day {
        match weekday {
            chrono::Weekday::Mon => &mut self.days[0],
            chrono::Weekday::Tue => &mut self.days[1],
            chrono::Weekday::Wed => &mut self.days[2],
            chrono::Weekday::Thu => &mut self.days[3],
            chrono::Weekday::Fri => &mut self.days[4],
            chrono::Weekday::Sat => &mut self.days[5],
            chrono::Weekday::Sun => &mut self.days[6],
        }
    }

    pub fn get_day(&self, weekday: chrono::Weekday) -> &Day {
        match weekday {
            chrono::Weekday::Mon => &self.days[0],
            chrono::Weekday::Tue => &self.days[1],
            chrono::Weekday::Wed => &self.days[2],
            chrono::Weekday::Thu => &self.days[3],
            chrono::Weekday::Fri => &self.days[4],
            chrono::Weekday::Sat => &self.days[5],
            chrono::Weekday::Sun => &self.days[6],
        }
    }

    pub fn next_week_id(&self) -> DateTime<Local> {
        self.id + chrono::Duration::days(7)
    }

    pub fn prev_week_id(&self) -> DateTime<Local> {
        self.id - chrono::Duration::days(7)
    }

    pub(crate) fn canonize(&mut self) {
        for day in self.days.iter_mut() {
            day.training.sort_by(|a, b| a.start_at().cmp(&b.start_at()));
        }
    }

    pub fn day_date(&self, weekday: chrono::Weekday) -> DateTime<Local> {
        self.id + chrono::Duration::days(weekday.num_days_from_monday() as i64)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Day {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub weekday: chrono::Weekday,
    pub training: Vec<Training>,
}

impl Day {
    pub fn new(week_day: chrono::Weekday) -> Day {
        Day {
            weekday: week_day,
            training: Vec::new(),
            id: ObjectId::new(),
        }
    }

    pub fn add_training(&mut self, training: Training) -> bool {
        let new_training_start_at = training.start_at();
        let new_training_end_at =
            training.start_at() + chrono::Duration::minutes(training.duration_min as i64);

        let conflict = self
            .training
            .iter()
            .map(|t| {
                (
                    t.start_at(),
                    t.start_at() + chrono::Duration::minutes(t.duration_min as i64),
                )
            })
            .any(|(start, end)| {
                (new_training_start_at >= start && new_training_start_at < end)
                    || (new_training_end_at > start && new_training_end_at <= end)
            });

        if !conflict {
            self.training.push(training);
            true
        } else {
            false
        }
    }
}
