use super::View;
use crate::{callback_data::Calldata as _, context::Context, state::Widget};
use async_trait::async_trait;
use chrono::{Datelike, NaiveDate, Weekday};
use day::DayView;
use eyre::eyre;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use storage::{
    schedule::model::{Day, Week},
    training::model::TrainingStatus,
};
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};
use teloxide::{prelude::Requester as _, types::Message};

pub mod day;

pub struct CalendarView {
    go_back: Option<Widget>,
    weed_id: NaiveDate,
}

impl CalendarView {
    pub fn new(id: NaiveDate, go_back: Option<Widget>) -> Self {
        Self {
            go_back,
            weed_id: id,
        }
    }
}

#[async_trait]
impl View for CalendarView {
    async fn show(&mut self, ctx: &mut Context) -> Result<(), eyre::Error> {
        let week = ctx.ledger.get_week(Some(self.weed_id)).await?;

        let (text, keymap) = render_week(
            &week,
            ctx.ledger.has_prev_week(&week),
            ctx.ledger.has_next_week(&week),
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
                self.weed_id = week;
                self.show(ctx).await?;
                Ok(None)
            }
            CalendarCallback::SelectDay(day) => {
                let view = DayView::new(
                    day,
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
    let now = chrono::Local::now().naive_local().date();
    for day in &week.days {
        if day.date < now {
            continue;
        }

        let mut row = vec![];
        let short = day.training.iter().map(|t| &t.short_name).join("/");

        let name = format!(
            "{} {} : {} {}",
            render_weekday(&day.date),
            day.date.format("%d.%m"),
            render_day_status(&day),
            short
        );

        row.push(InlineKeyboardButton::callback(
            name,
            CalendarCallback::SelectDay(day.date).to_data(),
        ));
        buttons = buttons.append_row(row);
    }

    let mut last_row = vec![];
    if has_prev {
        last_row.push(InlineKeyboardButton::callback(
            "⬅️",
            CalendarCallback::GoToWeek(week.prev_week_id()).to_data(),
        ));
    }

    if hes_next {
        last_row.push(InlineKeyboardButton::callback(
            "➡️",
            CalendarCallback::GoToWeek(week.next_week_id()).to_data(),
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

fn render_weekday(weekday: &NaiveDate) -> &'static str {
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
        if training.status != TrainingStatus::Full {
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

pub fn render_training_status(training: &TrainingStatus) -> &'static str {
    match training {
        TrainingStatus::Finished => "✔️",
        TrainingStatus::OpenToSignup => "🟢",
        TrainingStatus::ClosedToSignup => "🟠",
        TrainingStatus::InProgress => "🔵",
        TrainingStatus::Cancelled => "⛔",
        TrainingStatus::Full => "🟣",
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum CalendarCallback {
    GoToWeek(NaiveDate),
    SelectDay(NaiveDate),
    Back,
}
