use crate::training::model::Training;
use chrono::{DateTime, Datelike, Local, TimeZone as _, Utc, Weekday};
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
}

impl Day {
    pub fn new(day: DayId) -> Day {
        Day {
            weekday: day.local().weekday(),
            training: Vec::new(),
            id: ObjectId::new(),
            date_time: day.id(),
        }
    }

    pub fn day_id(&self) -> DayId {
        DayId(self.date_time)
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
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct WeekId(DateTime<Utc>);

impl WeekId {
    pub fn new(date_time: DateTime<Local>) -> Self {
        let date = date_time
            .date_naive()
            .week(Weekday::Mon)
            .first_day()
            .and_hms_opt(0, 0, 0)
            .unwrap();
        let local_date = Local.from_local_datetime(&date).earliest().unwrap();
        WeekId(local_date.with_timezone(&Utc))
    }

    pub fn local(&self) -> DateTime<Local> {
        self.0.with_timezone(&Local)
    }

    pub fn id(&self) -> DateTime<Utc> {
        self.0
    }

    pub fn next(&self) -> Self {
        WeekId(self.0 + chrono::Duration::days(7))
    }

    pub fn prev(&self) -> Self {
        WeekId(self.0 - chrono::Duration::days(7))
    }

    pub fn has_week(&self) -> bool {
        let now = Utc::now();
        let max_year = now.year() + 2;
        let current_year = self.0.year();
        current_year <= max_year && self.next().0 > now
    }

    pub fn day(&self, weekday: Weekday) -> DayId {
        let date = self.local() + chrono::Duration::days(weekday.num_days_from_monday() as i64);
        DayId(date.with_timezone(&Utc))
    }
}

impl Default for WeekId {
    fn default() -> Self {
        WeekId::new(Local::now())
    }
}

impl From<DateTime<Local>> for WeekId {
    fn from(date_time: DateTime<Local>) -> Self {
        WeekId::new(date_time)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct DayId(DateTime<Utc>);

impl DayId {
    pub fn new(date_time: DateTime<Local>) -> Self {
        let date = date_time.date().and_hms(0, 0, 0);
        DayId(date.with_timezone(&Utc))
    }

    pub fn local(&self) -> DateTime<Local> {
        self.0.with_timezone(&Local)
    }

    pub fn id(&self) -> DateTime<Utc> {
        self.0
    }

    pub fn week_day(&self) -> Weekday {
        self.local().weekday()
    }

    pub fn week_id(&self) -> WeekId {
        WeekId::new(self.local())
    }

    pub fn next(&self) -> Self {
        DayId(self.0 + chrono::Duration::days(1))
    }

    pub fn prev(&self) -> Self {
        DayId(self.0 - chrono::Duration::days(1))
    }
}

impl From<DateTime<Local>> for DayId {
    fn from(date_time: DateTime<Local>) -> Self {
        DayId::new(date_time)
    }
}

impl From<DateTime<Utc>> for DayId {
    fn from(date_time: DateTime<Utc>) -> Self {
        DayId::from(date_time.with_timezone(&Local))
    }
}

impl Default for DayId {
    fn default() -> Self {
        DayId::new(Local::now())
    }
}
