use eyre::eyre;
use teloxide::{
    prelude::Requester,
    types::{InlineKeyboardButton, InlineKeyboardMarkup},
};

use crate::state::State;

use super::ScheduleState;

pub enum ScheduleLendingCallback {
    MyTrainings,
    Schedule,
    FindTraining,
}

impl TryFrom<&str> for ScheduleLendingCallback {
    type Error = eyre::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "slc_my_trainings" => Ok(Self::MyTrainings),
            "slc_schedule" => Ok(Self::Schedule),
            "slc_find_training" => Ok(Self::FindTraining),
            _ => Err(eyre!("Unknown schedule lending callback")),
        }
    }
}

impl ScheduleLendingCallback {
    pub fn to_data(&self) -> String {
        match self {
            ScheduleLendingCallback::MyTrainings => "slc_my_trainings".to_owned(),
            ScheduleLendingCallback::Schedule => "slc_schedule".to_owned(),
            ScheduleLendingCallback::FindTraining => "slc_find_training".to_owned(),
        }
    }
}

pub fn render() -> (String, InlineKeyboardMarkup) {
    let msg = "📅  Подберем тренировку для вас:".to_owned();
    let mut keyboard = InlineKeyboardMarkup::default();
    keyboard = keyboard.append_row(vec![InlineKeyboardButton::callback(
        "🫶🏻 Мои тренировки",
        ScheduleLendingCallback::MyTrainings.to_data(),
    )]);
    keyboard = keyboard.append_row(vec![InlineKeyboardButton::callback(
        "📅  Расписание",
        "slc_find_training",
    )]);
    keyboard = keyboard.append_row(vec![InlineKeyboardButton::callback(
        "🔍 Найти тренировку",
        "slc_find_training",
    )]);

    (msg, keyboard)
}

pub(crate) async fn handle_message(
    bot: &teloxide::Bot,
    me: &storage::user::User,
    ledger: &ledger::Ledger,
    message: &teloxide::prelude::Message,
) -> Result<Option<State>, eyre::Error> {
    bot.delete_message(message.chat.id, message.id).await?;
    ScheduleState::Lending.into()
}

pub(crate) async fn handle_callback(
    bot: &teloxide::Bot,
    me: &storage::user::User,
    ledger: &ledger::Ledger,
    q: &teloxide::prelude::CallbackQuery,
) -> Result<Option<State>, eyre::Error> {
    todo!()
}
