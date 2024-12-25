use bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum Room {
    #[default]
    Adult,
    Child,
}

impl Room {
    pub fn id(&self) -> ObjectId {
        match self {
            Room::Adult => ObjectId::from_bytes(*b"adult0000000"),
            Room::Child => ObjectId::from_bytes(*b"child0000000"),
        }
    }
}

impl From<ObjectId> for Room {
    fn from(id: ObjectId) -> Self {
        match &id.bytes() {
            b"adult0000000" => Room::Adult,
            b"child0000000" => Room::Child,
            _ => Room::default(),
        }
    }
}