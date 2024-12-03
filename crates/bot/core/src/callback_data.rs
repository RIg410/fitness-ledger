use chrono::{DateTime, Datelike as _, Local, TimeZone as _, Timelike as _, Utc};
use log::error;
use model::ids::{DayId, WeekId};
use model::training::TrainingId;
use mongodb::bson::oid::ObjectId;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use teloxide::types::InlineKeyboardButton;

pub fn encode_data<T>(data: &T, type_id: u16) -> String
where
    T: Serialize + ?Sized,
{
    bs58::encode(bincode::serialize(&(data, type_id)).unwrap()).into_string()
}

pub fn decode_data<T>(data: &str) -> Result<(T, u16), eyre::Error>
where
    T: DeserializeOwned,
{
    Ok(bincode::deserialize(&bs58::decode(data).into_vec()?)?)
}

pub trait Calldata {
    fn to_data(&self) -> String;
    fn from_data(data: &str) -> Option<Self>
    where
        Self: Sized;

    fn button<N: Into<String>>(&self, name: N) -> InlineKeyboardButton {
        let data = self.to_data();
        if data.len() > 64 {
            let name = name.into();
            error!("Invalid callback data:{} [{}]", data, name);
            InlineKeyboardButton::callback(name, data)
        } else {
            InlineKeyboardButton::callback(name, data)
        }
    }
    fn btn_row<N: Into<String>>(&self, name: N) -> Vec<InlineKeyboardButton> {
        vec![self.button(name)]
    }
}

impl<T> Calldata for T
where
    T: Serialize + DeserializeOwned,
{
    fn to_data(&self) -> String {
        encode_data(self, type_id::<T>())
    }

    fn from_data(data: &str) -> Option<Self> {
        let (data, id) = decode_data(data).ok()?;
        if id != type_id::<T>() {
            return None;
        }
        Some(data)
    }
}

fn type_id<T>() -> u16 {
    let type_name = std::any::type_name::<T>();
    let mut hasher = DefaultHasher::new();
    type_name.hash(&mut hasher);
    (hasher.finish() % u16::MAX as u64) as u16
}

#[macro_export]
macro_rules! calldata {
    ($data:expr) => {
        if let Some(cb) = bot_core::callback_data::Calldata::from_data($data) {
            cb
        } else {
            return Ok(bot_core::widget::Jmp::Stay);
        }
    };
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct CallbackDateTime {
    year: i32,
    month: u8,
    day: u8,
    hour: u8,
    minute: u8,
    second: u8,
}

impl From<DateTime<Local>> for CallbackDateTime {
    fn from(date: DateTime<Local>) -> Self {
        Self {
            year: date.year(),
            month: date.month() as u8,
            day: date.day() as u8,
            hour: date.hour() as u8,
            minute: date.minute() as u8,
            second: date.second() as u8,
        }
    }
}

impl From<DateTime<Utc>> for CallbackDateTime {
    fn from(date: DateTime<Utc>) -> Self {
        Self {
            year: date.year(),
            month: date.month() as u8,
            day: date.day() as u8,
            hour: date.hour() as u8,
            minute: date.minute() as u8,
            second: date.second() as u8,
        }
    }
}

impl From<CallbackDateTime> for WeekId {
    fn from(date: CallbackDateTime) -> Self {
        let local = DateTime::<Local>::from(date);
        WeekId::new(local)
    }
}

impl From<CallbackDateTime> for DayId {
    fn from(date: CallbackDateTime) -> Self {
        let local = DateTime::<Local>::from(date);
        DayId::from(local)
    }
}

impl From<CallbackDateTime> for DateTime<Local> {
    fn from(date: CallbackDateTime) -> Self {
        Local
            .with_ymd_and_hms(
                date.year,
                date.month as u32,
                date.day as u32,
                date.hour as u32,
                date.minute as u32,
                date.second as u32,
            )
            .earliest()
            .unwrap()
    }
}

impl From<CallbackDateTime> for DateTime<Utc> {
    fn from(date: CallbackDateTime) -> Self {
        Utc.with_ymd_and_hms(
            date.year,
            date.month as u32,
            date.day as u32,
            date.hour as u32,
            date.minute as u32,
            date.second as u32,
        )
        .earliest()
        .unwrap()
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct TrainingIdCallback {
    start_at: CallbackDateTime,
    room: [u8; 12],
}

impl From<TrainingId> for TrainingIdCallback {
    fn from(id: TrainingId) -> Self {
        TrainingIdCallback {
            start_at: CallbackDateTime::from(id.start_at),
            room: id.room.bytes(),
        }
    }
}

impl From<TrainingIdCallback> for TrainingId {
    fn from(id: TrainingIdCallback) -> Self {
        Self {
            start_at: DateTime::<Utc>::from(id.start_at),
            room: ObjectId::from_bytes(id.room),
        }
    }
}
