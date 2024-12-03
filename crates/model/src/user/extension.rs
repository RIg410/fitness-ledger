use bson::oid::ObjectId;
use chrono::{DateTime, Datelike as _, Local};
use serde::{Deserialize, Serialize};
use std::fmt::{self, Display, Formatter};

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct UserExtension {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub birthday: Option<Birthday>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Birthday {
    day: u32,
    month: u32,
    year: i32,
}

impl Birthday {
    pub fn new(dt: DateTime<Local>) -> Birthday {
        Birthday {
            day: dt.day(),
            month: dt.month(),
            year: dt.year(),
        }
    }
}

impl Display for Birthday {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:02}.{:02}.{}", self.day, self.month, self.year)
    }
}
