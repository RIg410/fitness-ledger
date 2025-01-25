use chrono::{DateTime, Datelike, Local, Timelike as _, Utc};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

use crate::{
    decimal::Decimal,
    ids::DayId,
    program::{Program, TrainingType},
    rooms::Room,
    slot::Slot,
};

pub const CLOSE_SING_UP: u32 = 3 * 60; // 3 hours

#[derive(Debug, Serialize, Deserialize, Clone)]
#[non_exhaustive]
pub struct Training {
    #[serde(default = "default_room_id")]
    room: ObjectId,
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub proto_id: ObjectId,
    pub name: String,
    pub description: String,
    #[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    start_at: DateTime<Utc>,
    pub duration_min: u32,
    pub instructor: ObjectId,
    pub clients: Vec<ObjectId>,
    pub capacity: u32,
    pub is_one_time: bool,
    #[serde(default)]
    pub is_canceled: bool,
    #[serde(default)]
    #[serde(rename = "is_finished")]
    pub is_processed: bool,
    #[serde(default)]
    pub statistics: Option<Statistics>,
    #[serde(default)]
    pub notified: Notified,
    #[serde(default)]
    pub keep_open: bool,
    #[serde(default)]
    pub tp: TrainingType,
}

impl Training {
    pub fn new(
        proto_id: ObjectId,
        name: String,
        description: String,
        start_at: DateTime<Utc>,
        duration_min: u32,
        instructor: ObjectId,
        capacity: u32,
        is_one_time: bool,
        tp: TrainingType,
        room: ObjectId,
    ) -> Training {
        Training {
            id: ObjectId::new(),
            proto_id,
            name,
            description,
            start_at,
            duration_min,
            instructor,
            clients: Vec::new(),
            capacity,
            is_one_time,
            is_canceled: false,
            is_processed: false,
            statistics: None,
            notified: Default::default(),
            keep_open: false,
            tp,
            room,
        }
    }

    pub fn new_personal(
        start_at: DateTime<Local>,
        room: ObjectId,
        instructor: ObjectId,
        duration_min: u32,
        name: String,
        instructor_description: String,
    ) -> Training {
        Training {
            id: ObjectId::new(),
            proto_id: ObjectId::from_bytes([0; 12]),
            name,
            description: instructor_description,
            start_at: start_at.with_timezone(&Utc),
            duration_min,
            instructor,
            clients: vec![],
            capacity: 1,
            is_one_time: true,
            is_canceled: false,
            is_processed: false,
            statistics: None,
            notified: Default::default(),
            keep_open: false,
            tp: TrainingType::Personal { is_free: false },
            room,
        }
    }

    pub fn new_group(
        program: Program,
        start_at: DateTime<Local>,
        instructor: ObjectId,
        is_one_time: bool,
        room: ObjectId,
    ) -> Training {
        Training {
            id: ObjectId::new(),
            proto_id: program.id,
            name: program.name,
            description: program.description,
            start_at: start_at.with_timezone(&Utc),
            duration_min: program.duration_min,
            instructor,
            clients: Vec::new(),
            capacity: program.capacity,
            is_one_time,
            is_canceled: false,
            is_processed: false,
            statistics: None,
            notified: Default::default(),
            keep_open: false,
            tp: program.tp,
            room,
        }
    }

    pub fn start_at_utc(&self) -> DateTime<Utc> {
        self.start_at
    }

    pub fn with_day_and_training(day: DayId, training: Training) -> Training {
        let start_time = training.get_slot().start_at();
        let start_date = day.local();
        let start_at = start_date
            .with_hour(start_time.hour())
            .unwrap()
            .with_minute(start_time.minute())
            .unwrap();

        Training {
            id: training.id,
            proto_id: training.proto_id,
            name: training.name,
            description: training.description,
            start_at: start_at.with_timezone(&Utc),
            duration_min: training.duration_min,
            instructor: training.instructor,
            clients: vec![],
            capacity: training.capacity,
            is_one_time: training.is_one_time,
            is_canceled: false,
            is_processed: false,
            statistics: None,
            notified: Default::default(),
            keep_open: false,
            tp: training.tp,
            room: training.room,
        }
    }

    pub fn get_slot(&self) -> Slot {
        Slot::new(self.start_at, self.duration_min, self.room)
    }

    pub fn status(&self, now: DateTime<Local>) -> TrainingStatus {
        if self.is_canceled {
            TrainingStatus::Cancelled
        } else {
            let start_at = self.get_slot().start_at();
            let end_at = start_at + chrono::Duration::minutes(self.duration_min as i64);
            if end_at < now {
                TrainingStatus::Finished
            } else if start_at < now {
                TrainingStatus::InProgress
            } else if start_at - chrono::Duration::minutes(CLOSE_SING_UP as i64) < now {
                if self.clients.is_empty() && !self.keep_open {
                    TrainingStatus::ClosedToSignup
                } else {
                    TrainingStatus::OpenToSignup {
                        close_sign_out: true,
                    }
                }
            } else {
                TrainingStatus::OpenToSignup {
                    close_sign_out: false,
                }
            }
        }
    }

    pub fn is_full(&self) -> bool {
        self.clients.len() as u32 >= self.capacity
    }

    pub fn set_date(&mut self, week_date: DateTime<Local>) -> Result<(), eyre::Error> {
        self.start_at = self
            .start_at
            .with_day(week_date.day())
            .ok_or_else(|| eyre::eyre!("Invalid day"))?
            .with_year(week_date.year())
            .ok_or_else(|| eyre::eyre!("Invalid day"))?
            .with_month(week_date.month())
            .ok_or_else(|| eyre::eyre!("Invalid day"))?;
        Ok(())
    }

    pub fn day_id(&self) -> DayId {
        DayId::from(self.start_at)
    }

    pub fn id(&self) -> TrainingId {
        TrainingId {
            start_at: self.start_at,
            room: self.room,
        }
    }

    pub fn room(&self) -> ObjectId {
        self.room
    }

    pub fn is_group(&self) -> bool {
        self.tp.is_group()
    }

    pub fn is_personal(&self) -> bool {
        self.tp.is_personal()
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Copy)]
pub enum TrainingStatus {
    OpenToSignup { close_sign_out: bool },
    ClosedToSignup,
    InProgress,
    Cancelled,
    Finished,
}

impl TrainingStatus {
    pub fn can_be_canceled(&self) -> bool {
        matches!(
            self,
            TrainingStatus::OpenToSignup { .. } | TrainingStatus::ClosedToSignup
        )
    }

    pub fn can_be_uncanceled(&self) -> bool {
        matches!(self, TrainingStatus::Cancelled)
    }

    pub fn can_sign_out(&self) -> bool {
        if let TrainingStatus::OpenToSignup { close_sign_out } = self {
            !close_sign_out
        } else {
            false
        }
    }

    pub fn can_sign_in(&self) -> bool {
        matches!(self, TrainingStatus::OpenToSignup { .. })
    }
}

#[derive(Debug, Clone)]
pub enum Filter {
    Client(ObjectId),
    Instructor(ObjectId),
    Program(ObjectId),
}

impl Filter {
    pub fn is_match(&self, training: &Training) -> bool {
        match self {
            Filter::Client(client) => training.clients.contains(client),
            Filter::Instructor(instructor) => training.instructor == *instructor,
            Filter::Program(program) => training.proto_id == *program,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Statistics {
    pub earned: Decimal,
    pub couch_rewards: Decimal,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Notified {
    None {},
    Tomorrow {},
    ByHours(Vec<ObjectId>),
}

impl Notified {
    pub fn is_notified(&self) -> bool {
        !matches!(self, Notified::None {})
    }
}

impl Default for Notified {
    fn default() -> Self {
        Notified::None {}
    }
}

fn default_room_id() -> ObjectId {
    Room::Adult.id()
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TrainingId {
    pub start_at: DateTime<Utc>,
    pub room: ObjectId,
}

impl TrainingId {
    pub fn day_id(&self) -> DayId {
        DayId::from(self.start_at)
    }

    pub fn start_at(&self) -> DateTime<Local> {
        self.start_at.with_timezone(&Local)
    }
}
