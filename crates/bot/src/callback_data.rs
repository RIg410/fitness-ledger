use serde::de::DeserializeOwned;
use serde::Serialize;

pub fn encode_data<T: ?Sized>(data: &T) -> String
where
    T: Serialize,
{
    hex::encode(bincode::serialize(data).unwrap())
}

pub fn decode_data<T>(data: &str) -> Result<T, eyre::Error>
where
    T: DeserializeOwned,
{
    Ok(bincode::deserialize(&hex::decode(data)?)?)
}

pub trait Calldata {
    fn to_data(&self) -> String;
    fn from_data(data: &str) -> Result<Self, eyre::Error>
    where
        Self: Sized;
}

impl<T> Calldata for T
where
    T: Serialize + DeserializeOwned,
{
    fn to_data(&self) -> String {
        encode_data(self)
    }

    fn from_data(data: &str) -> Result<Self, eyre::Error> {
        decode_data(data)
    }
}
