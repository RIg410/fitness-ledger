use bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

use crate::decimal::Decimal;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Program {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub name: String,
    pub description: String,
    pub duration_min: u32,
    pub capacity: u32,
    #[serde(default)]
    pub version: u64,
    #[serde(default)]
    pub tp: TrainingType,
    #[serde(default = "default_visible")]
    pub visible: bool,
}

fn default_visible() -> bool {
    true
}

impl Default for Program {
    fn default() -> Self {
        Program {
            id: ObjectId::new(),
            name: String::new(),
            description: String::new(),
            duration_min: 0,
            capacity: 0,
            version: 0,
            tp: TrainingType::Group { is_free: false },
            visible: true,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Hash, Eq)]
pub enum TrainingType {
    Group { is_free: bool },
    Personal { is_free: bool },
    SubRent { is_free: bool, price: Decimal },
}

impl TrainingType {
    pub fn set_is_free(&mut self, is_free: bool) {
        match self {
            TrainingType::Group { is_free: ref mut f } => *f = is_free,
            TrainingType::Personal { is_free: ref mut f } => *f = is_free,
            TrainingType::SubRent {
                is_free: ref mut f,
                price: _,
            } => *f = is_free,
        }
    }

    pub fn is_group(&self) -> bool {
        matches!(self, TrainingType::Group { .. })
    }

    pub fn is_personal(&self) -> bool {
        matches!(self, TrainingType::Personal { .. })
    }

    pub fn is_sub_rent(&self) -> bool {
        matches!(self, TrainingType::SubRent { .. })
    }

    pub fn is_free(&self) -> bool {
        match self {
            TrainingType::Group { is_free } => *is_free,
            TrainingType::Personal { is_free } => *is_free,
            TrainingType::SubRent { is_free, price: _ } => *is_free,
        }
    }

    pub fn is_not_free(&self) -> bool {
        !self.is_free()
    }
}

impl Default for TrainingType {
    fn default() -> Self {
        TrainingType::Group { is_free: false }
    }
}
