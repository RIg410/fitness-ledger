use super::ScheduleState;
use crate::process::Origin;
use crate::state::State;
use chrono::NaiveDate;
use day::go_to_day;
use day::DayState;
use eyre::eyre;
use eyre::Result;
use ledger::Ledger;
use storage::user::User;
use teloxide::payloads::EditMessageTextSetters as _;
use teloxide::prelude::Requester;
use teloxide::Bot;
use week::render_week;

mod day;
mod plan_training;
mod week;

#[derive(Clone, Debug)]
pub enum CalendarState {
    Lending(Origin),
    Day(DayState),
}

impl CalendarState {
    pub fn origin(&self) -> &Origin {
        match self {
            CalendarState::Lending(origin) => origin,
            CalendarState::Day(DayState::Lending(day)) => &day.origin,
            CalendarState::Day(DayState::AddingTraining(state)) => state.origin(),
        }
    }
}

impl From<CalendarState> for Result<Option<State>> {
    fn from(state: CalendarState) -> Self {
        Ok(Some(State::Schedule(ScheduleState::Calendar(state))))
    }
}

#[derive(Clone, Debug)]
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
    user: &storage::user::User,
    ledger: &ledger::Ledger,
    message: &teloxide::prelude::Message,
    state: CalendarState,
) -> Result<Option<State>> {
    match state {
        CalendarState::Lending(_) => {
            bot.delete_message(message.chat.id, message.id).await?;
            state.into()
        }
        CalendarState::Day(state) => day::handle_message(bot, user, ledger, message, state).await,
    }
}

pub(crate) async fn handle_callback(
    bot: &teloxide::Bot,
    me: &storage::user::User,
    ledger: &ledger::Ledger,
    q: &teloxide::prelude::CallbackQuery,
    state: CalendarState,
) -> Result<Option<State>> {
    let data = q
        .data
        .as_ref()
        .ok_or_else(|| eyre!("No data in callback"))?;
    if let Ok(callback) = ScheduleCalendarCallback::try_from(data.as_str()) {
        let origin = state.origin();
        match callback {
            ScheduleCalendarCallback::GoToWeek(week_id) => {
                go_to_calendar(bot, me, ledger, origin.clone(), Some(week_id)).await
            }
            ScheduleCalendarCallback::SelectDay(day) => {
                go_to_day(bot, me, ledger, origin.clone(), day).await
            }
        }
    } else {
        match state {
            CalendarState::Lending(origin) => CalendarState::Lending(origin).into(),
            CalendarState::Day(state) => day::handle_callback(bot, me, ledger, q, state).await,
        }
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

    ScheduleState::Calendar(CalendarState::Lending(origin)).into()
}
