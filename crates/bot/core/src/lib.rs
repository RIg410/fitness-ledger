use std::str::FromStr;

use callback_data::Calldata;
use mongodb::bson::oid::ObjectId;
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};

pub mod bot;
pub mod callback_data;
pub mod context;
pub mod err;
pub mod handlers;
pub mod script;
pub mod state;
pub mod widget;

const ERROR: &str = "Что-то пошло не так. Пожалуйста, попробуйте позже.";

const HOME_DESCRIPTION: &str = "🏠 Меню";
const HOME_NAME: &str = "/start";

const BACK_DESCRIPTION: &str = "🔙 Назад";
const BACK_NAME: &str = "/back";

pub(crate) fn sys_button(keymap: InlineKeyboardMarkup, can_back: bool) -> InlineKeyboardMarkup {
    let mut row = vec![];
    if can_back {
        row.push(InlineKeyboardButton::callback(BACK_DESCRIPTION, BACK_NAME));
    }
    row.push(InlineKeyboardButton::callback(HOME_DESCRIPTION, HOME_NAME));
    keymap.append_row(row)
}

#[derive(Debug)]
pub enum CommonLocation {
    Profile(ObjectId),
    Request(ObjectId),
}

impl CommonLocation {
    pub fn is_cmd(msg: &str) -> bool {
        msg.starts_with("/cl/")
    }

    pub fn button(&self) -> InlineKeyboardButton {
        let name = match self {
            CommonLocation::Profile(_) => "👤 Профиль",
            CommonLocation::Request(_) => "📝 Заявка",
        };
        InlineKeyboardButton::callback(name, self.to_data())
    }
}

impl Calldata for CommonLocation {
    fn to_data(&self) -> String {
        let (tp, id) = match self {
            CommonLocation::Profile(id) => ("usr", id.to_hex()),
            CommonLocation::Request(id) => ("req", id.to_hex()),
        };
        format!("/cl/{}/{}", tp, id)
    }

    fn from_data(data: &str) -> Option<Self>
    where
        Self: Sized,
    {
        let mut parts = data.split('/');
        let _ = parts.next()?;
        let cl = parts.next()?;
        if cl != "cl" {
            return None;
        }

        let tp = parts.next()?;
        match tp {
            "usr" => {
                let id = parts.next()?;
                Some(CommonLocation::Profile(ObjectId::from_str(id).ok()?))
            }
            "req" => {
                let id = parts.next()?;
                Some(CommonLocation::Request(ObjectId::from_str(id).ok()?))
            }
            _ => None,
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_to_data_profile() {
        let id = ObjectId::new();
        let location = CommonLocation::Profile(id.clone());
        let data = location.to_data();
        assert_eq!(data, format!("/cl/usr/{}", id.to_hex()));
    }

    #[test]
    fn test_to_data_request() {
        let id = ObjectId::new();
        let location = CommonLocation::Request(id.clone());
        let data = location.to_data();
        assert_eq!(data, format!("/cl/req/{}", id.to_hex()));
    }

    #[test]
    fn test_from_data_profile() {
        let id = ObjectId::new();
        let data = format!("/cl/usr/{}", id.to_hex());
        let location = CommonLocation::from_data(&data).unwrap();
        match location {
            CommonLocation::Profile(profile_id) => assert_eq!(profile_id, id),
            _ => panic!("Expected CommonLocation::Profile"),
        }
    }

    #[test]
    fn test_from_data_request() {
        let id = ObjectId::new();
        let data = format!("/cl/req/{}", id.to_hex());
        let location = CommonLocation::from_data(&data).unwrap();
        match location {
            CommonLocation::Request(request_id) => assert_eq!(request_id, id),
            _ => panic!("Expected CommonLocation::Request"),
        }
    }

    #[test]
    fn test_from_data_invalid() {
        let data = "/cl/invalid/12345";
        let location = CommonLocation::from_data(data);
        assert!(location.is_none());
    }

    #[test]
    fn test_from_data_invalid_format() {
        let data = "/invalid_format";
        let location = CommonLocation::from_data(data);
        assert!(location.is_none());
    }
}
