use super::ScheduleState;
use crate::process::Origin;
use crate::state::State;
use chrono::NaiveDate;
use eyre::eyre;
use eyre::Result;
use ledger::Ledger;
use rendering::render_week;
use storage::user::User;
use teloxide::payloads::EditMessageTextSetters as _;
use teloxide::prelude::Requester;
use teloxide::Bot;
mod rendering;

pub enum ScheduleCalendarCallback {
    GoToWeek(NaiveDate),
    SelectDay(NaiveDate),
}

impl ScheduleCalendarCallback {
    pub fn to_data(&self) -> String {
        match self {
            ScheduleCalendarCallback::GoToWeek(date) => {
                format!("scc_goto:{}", date.format("%Y-%m-%d"))
            }
            ScheduleCalendarCallback::SelectDay(date) => {
                format!("scc_select:{}", date.format("%Y-%m-%d"))
            }
        }
    }
}

impl TryFrom<&str> for ScheduleCalendarCallback {
    type Error = eyre::Error;

    fn try_from(value: &str) -> std::result::Result<Self, Self::Error> {
        let parts: Vec<&str> = value.split(':').collect();
        if parts.len() != 2 {
            return Err(eyre!("Invalid ScheduleCalendarCallback"));
        }

        let date = NaiveDate::parse_from_str(parts[1], "%Y-%m-%d")?;
        match parts[0] {
            "scc_goto" => Ok(Self::GoToWeek(date)),
            "scc_select" => Ok(Self::SelectDay(date)),
            _ => Err(eyre!("Invalid ScheduleCalendarCallback")),
        }
    }
}

pub(crate) async fn handle_message(
    bot: &teloxide::Bot,
    _: &storage::user::User,
    _: &ledger::Ledger,
    message: &teloxide::prelude::Message,
    origin: Origin,
) -> Result<Option<State>> {
    bot.delete_message(message.chat.id, message.id).await?;
    ScheduleState::Calendar(origin).into()
}

pub(crate) async fn handle_callback(
    bot: &teloxide::Bot,
    me: &storage::user::User,
    ledger: &ledger::Ledger,
    q: &teloxide::prelude::CallbackQuery,
    origin: Origin,
) -> Result<Option<State>> {
    let data = q
        .data
        .as_ref()
        .ok_or_else(|| eyre!("No data in callback"))?;
    let callback = ScheduleCalendarCallback::try_from(data.as_str())?;

    match callback {
        ScheduleCalendarCallback::GoToWeek(week_id) => {
            go_to_calendar(bot, me, ledger, origin, Some(week_id)).await
        }
        ScheduleCalendarCallback::SelectDay(day) => todo!(),
    }
}

pub async fn go_to_calendar(
    bot: &Bot,
    _: &User,
    ledger: &Ledger,
    origin: Origin,
    week: Option<NaiveDate>,
) -> Result<Option<State>> {
    let week = ledger.get_week(week).await?;

    let (text, keymap) = render_week(
        &week,
        ledger.has_prev_week(&week),
        ledger.has_next_week(&week),
    );
    bot.edit_message_text(origin.chat_id, origin.message_id, text)
        .parse_mode(teloxide::types::ParseMode::MarkdownV2)
        .reply_markup(keymap)
        .await?;

    ScheduleState::Calendar(origin).into()
}
