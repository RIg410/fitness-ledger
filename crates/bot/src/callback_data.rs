use serde::de::DeserializeOwned;
use serde::Serialize;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use teloxide::types::InlineKeyboardButton;

pub fn encode_data<T>(data: &T, type_id: u32) -> String
where
    T: Serialize + ?Sized,
{
    hex::encode(bincode::serialize(&(data, type_id)).unwrap())
}

pub fn decode_data<T>(data: &str) -> Result<(T, u32), eyre::Error>
where
    T: DeserializeOwned,
{
    Ok(bincode::deserialize(&hex::decode(data)?)?)
}

pub trait Calldata {
    fn to_data(&self) -> String;
    fn from_data(data: &str) -> Option<Self>
    where
        Self: Sized;

    fn button(&self, name: String) -> InlineKeyboardButton {
        InlineKeyboardButton::callback(name, self.to_data())
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

fn type_id<T>() -> u32 {
    let type_name = std::any::type_name::<T>();
    let mut hasher = DefaultHasher::new();
    type_name.hash(&mut hasher);
    (hasher.finish() % u32::MAX as u64) as u32
}
