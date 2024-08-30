use super::{CalendarState, ScheduleCalendarCallback};
use crate::{
    process::{schedule_menu::ScheduleState, Origin},
    state::State,
};
use chrono::{Duration, NaiveDate};
use eyre::Result;
use ledger::Ledger;
use storage::{schedule::model::Day, user::User};
use teloxide::{
    payloads::EditMessageTextSetters as _, prelude::Requester as _, types::{InlineKeyboardButton, InlineKeyboardMarkup},
    Bot,
};

#[derive(Clone, Debug)]
pub enum DayState {
    Lending(DayLending),
}

#[derive(Clone, Debug)]
pub struct DayLending {
    pub origin: Origin,
    pub date: NaiveDate,
}

impl From<DayState> for Result<Option<State>> {
    fn from(state: DayState) -> Self {
        Ok(Some(State::Schedule(ScheduleState::Calendar(
            CalendarState::Day(state),
        ))))
    }
}

pub async fn go_to_day(
    bot: &Bot,
    me: &User,
    ledger: &Ledger,
    origin: Origin,
    day: NaiveDate,
) -> Result<Option<State>> {
    println!("{:?}", day);
    let day = ledger.get_day(day).await?;
    let (msg, keymap) = render_day(me, ledger, &day);
    bot.edit_message_text(origin.chat_id, origin.message_id, msg)
        .reply_markup(keymap)
        .parse_mode(teloxide::types::ParseMode::MarkdownV2)
        .await?;
    DayState::Lending(DayLending {
        origin: origin,
        date: day.date,
    })
    .into()
}

pub(crate) async fn handle_message(
    bot: &Bot,
    user: &User,
    ledger: &Ledger,
    message: &teloxide::prelude::Message,
    state: DayState,
) -> std::result::Result<Option<State>, eyre::Error> {
    todo!()
}

pub(crate) async fn handle_callback(
    bot: &Bot,
    me: &User,
    ledger: &Ledger,
    q: &teloxide::prelude::CallbackQuery,
    state: DayState,
) -> std::result::Result<Option<State>, eyre::Error> {
    match state {
        DayState::Lending(lending) => {
            go_to_day(bot, me, ledger, lending.origin, lending.date).await
        }
    }
}

fn render_day(me: &User, ledger: &Ledger, day: &Day) -> (String, InlineKeyboardMarkup) {
    let msg = format!("📅  Расписание на *{}*:", day.date.format("%d\\.%m\\.%Y"));
    let mut keymap = InlineKeyboardMarkup::default();

    let mut nav_row = vec![];
    let now = chrono::Local::now().naive_local().date();
    if now < day.date {
        nav_row.push(InlineKeyboardButton::callback(
            "⬅️",
            ScheduleCalendarCallback::SelectDay(day.date - Duration::days(1)).to_data(),
        ));
    }

    if ledger.has_week(day.date + Duration::days(1)) {
        nav_row.push(InlineKeyboardButton::callback(
            "➡️",
            ScheduleCalendarCallback::SelectDay(day.date + Duration::days(1)).to_data(),
        ));
    }
    keymap = keymap.append_row(nav_row);

    keymap = keymap.append_row(vec![InlineKeyboardButton::callback(
        "📅",
        ScheduleCalendarCallback::GoToWeek(day.date).to_data(),
    )]);
    (msg, keymap)
}
