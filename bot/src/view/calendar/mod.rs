use super::View;
use crate::{callback_data::Calldata as _, context::Context, state::Widget};
use async_trait::async_trait;
use chrono::{DateTime, Datelike, Local, TimeZone, Timelike as _, Weekday};
use day::DayView;
use serde::{Deserialize, Serialize};
use storage::{
    calendar::{
        day_id,
        model::{Day, Week},
    },
    training::model::TrainingStatus,
};
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};
use teloxide::{prelude::Requester as _, types::Message};

pub mod day;
pub mod training;

pub struct CalendarView {
    go_back: Option<Widget>,
    weed_id: DateTime<Local>,
}

impl CalendarView {
    pub fn new(id: DateTime<Local>, go_back: Option<Widget>) -> Self {
        Self {
            go_back,
            weed_id: id,
        }
    }
}

#[async_trait]
impl View for CalendarView {
    async fn show(&mut self, ctx: &mut Context) -> Result<(), eyre::Error> {
        let week = ctx.ledger.calendar.get_week(Some(self.weed_id)).await?;

        let (text, keymap) = render_week(
            &week,
            ctx.ledger.calendar.has_prev_week(&week),
            ctx.ledger.calendar.has_next_week(&week),
            self.go_back.is_some(),
        );
        ctx.edit_origin(&text, keymap).await?;
        Ok(())
    }

    async fn handle_message(
        &mut self,
        ctx: &mut Context,
        message: &Message,
    ) -> Result<Option<Widget>, eyre::Error> {
        ctx.bot.delete_message(message.chat.id, message.id).await?;
        Ok(None)
    }

    async fn handle_callback(
        &mut self,
        ctx: &mut Context,
        data: &str,
    ) -> Result<Option<Widget>, eyre::Error> {
        match CalendarCallback::from_data(data)? {
            CalendarCallback::GoToWeek(week) => {
                self.weed_id = week.into();
                self.show(ctx).await?;
                Ok(None)
            }
            CalendarCallback::SelectDay(day) => {
                let view = DayView::new(
                    day.into(),
                    Some(Box::new(CalendarView::new(
                        self.weed_id,
                        self.go_back.take(),
                    ))),
                );
                Ok(Some(Box::new(view)))
            }
            CalendarCallback::Back => {
                if let Some(widget) = self.go_back.take() {
                    Ok(Some(widget))
                } else {
                    Ok(None)
                }
            }
        }
    }
}

pub fn render_week(
    week: &Week,
    has_prev: bool,
    hes_next: bool,
    has_back: bool,
) -> (String, InlineKeyboardMarkup) {
    let msg = format!(
        "
📅  Расписание на неделю с *{}* по *{}*:
➖➖➖➖➖➖➖➖➖➖➖➖➖➖➖➖➖
🟢 \\- открыта запись 
🟣 \\- мест нет
✔️  \\- завершено
🟠 \\- запись закрыта
🌴 \\- нет тренировок
➖➖➖➖➖➖➖➖➖➖➖➖➖➖➖➖➖
        ",
        week.id.format("%d\\.%m\\.%Y"),
        (week.id + chrono::Duration::days(6)).format("%d\\.%m\\.%Y")
    );

    let mut buttons = InlineKeyboardMarkup::default();
    let now = day_id(chrono::Local::now()).unwrap_or_else(|| chrono::Local::now());

    for (day, date) in week.days() {
        if date < now {
            continue;
        }

        let mut row = vec![];
        let name = format!(
            "{} {} : {}: тренировки - {}",
            render_weekday(&date),
            date.format("%d.%m"),
            render_day_status(&day),
            day.training.len()
        );

        row.push(InlineKeyboardButton::callback(
            name,
            CalendarCallback::SelectDay(date.into()).to_data(),
        ));
        buttons = buttons.append_row(row);
    }

    let mut last_row = vec![];
    if has_prev {
        last_row.push(InlineKeyboardButton::callback(
            "⬅️ предыдущая неделя",
            CalendarCallback::GoToWeek(week.prev_week_id().into()).to_data(),
        ));
    }

    if hes_next {
        last_row.push(InlineKeyboardButton::callback(
            "➡️ cледующая неделя",
            CalendarCallback::GoToWeek(week.next_week_id().into()).to_data(),
        ));
    }
    buttons = buttons.append_row(last_row);

    if has_back {
        buttons = buttons.append_row(vec![InlineKeyboardButton::callback(
            "Назад",
            CalendarCallback::Back.to_data(),
        )]);
    }

    (msg, buttons)
}

fn render_weekday(weekday: &DateTime<Local>) -> &'static str {
    match weekday.weekday() {
        Weekday::Mon => "Пн",
        Weekday::Tue => "Вт",
        Weekday::Wed => "Ср",
        Weekday::Thu => "Чт",
        Weekday::Fri => "Пт",
        Weekday::Sat => "Сб",
        Weekday::Sun => "Вс",
    }
}

fn render_day_status(day: &Day) -> &'static str {
    if day.training.is_empty() {
        return "нет занятий 🌴";
    }
    let mut full = true;
    let mut finished = true;
    for training in &day.training {
        if training.status == TrainingStatus::OpenToSignup {
            return "🟢";
        }
        if !training.is_full() {
            full = false;
        }
        if training.status != TrainingStatus::Finished {
            finished = false;
        }
    }

    if finished {
        "✔️"
    } else if full {
        "🟣"
    } else {
        "🟠"
    }
}

pub fn render_training_status(training: &TrainingStatus, is_full: bool) -> &'static str {
    if is_full {
        return "🟣";
    }
    match training {
        TrainingStatus::Finished => "✔️",
        TrainingStatus::OpenToSignup => "🟢",
        TrainingStatus::ClosedToSignup => "🟠",
        TrainingStatus::InProgress => "🔵",
        TrainingStatus::Cancelled => "⛔",
    }
}

#[derive(Debug, Serialize, Deserialize)]
enum CalendarCallback {
    GoToWeek(CallbackDateTime),
    SelectDay(CallbackDateTime),
    Back,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CallbackDateTime {
    year: i32,
    month: u8,
    day: u8,
    hour: u8,
    minute: u8,
    second: u8,
}

impl From<DateTime<Local>> for CallbackDateTime {
    fn from(date: DateTime<Local>) -> Self {
        Self {
            year: date.year(),
            month: date.month() as u8,
            day: date.day() as u8,
            hour: date.hour() as u8,
            minute: date.minute() as u8,
            second: date.second() as u8,
        }
    }
}

impl From<CallbackDateTime> for DateTime<Local> {
    fn from(date: CallbackDateTime) -> Self {
        Local
            .with_ymd_and_hms(
                date.year,
                date.month as u32,
                date.day as u32,
                date.hour as u32,
                date.minute as u32,
                date.second as u32,
            )
            .earliest()
            .unwrap()
    }
}
