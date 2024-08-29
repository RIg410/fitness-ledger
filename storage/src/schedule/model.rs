use crate::date_time::{naive_date_deserialize, naive_date_serialize};
use crate::training::model::Training;
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Week {
    #[serde(serialize_with = "naive_date_serialize")]
    #[serde(deserialize_with = "naive_date_deserialize")]
    pub id: NaiveDate,
    pub days: [Day; 7],
}

impl Week {
    pub fn new(first_date: NaiveDate) -> Week {
        Week {
            id: first_date,
            days: [
                Day::new(first_date),
                Day::new(first_date + chrono::Duration::days(1)),
                Day::new(first_date + chrono::Duration::days(2)),
                Day::new(first_date + chrono::Duration::days(3)),
                Day::new(first_date + chrono::Duration::days(4)),
                Day::new(first_date + chrono::Duration::days(5)),
                Day::new(first_date + chrono::Duration::days(6)),
            ],
        }
    }

    pub fn next_week_id(&self) -> NaiveDate {
        self.id + chrono::Duration::days(7)
    }

    pub fn prev_week_id(&self) -> NaiveDate {
        self.id - chrono::Duration::days(7)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Day {
    #[serde(serialize_with = "naive_date_serialize")]
    #[serde(deserialize_with = "naive_date_deserialize")]
    pub date: NaiveDate,
    pub training: Vec<Training>,
}

impl Day {
    pub fn new(date: NaiveDate) -> Day {
        Day {
            date,
            training: Vec::new(),
        }
    }
}
