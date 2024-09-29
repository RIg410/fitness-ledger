use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};

pub mod callback_data;
pub mod context;
pub mod handlers;
pub mod script;
pub mod state;
pub mod widget;
pub mod bot;

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

