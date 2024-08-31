use super::{render_training_status, View};
use crate::callback_data::Calldata as _;
use crate::{context::Context, state::Widget};
use async_trait::async_trait;
use chrono::{Duration, NaiveDate};
use eyre::eyre;
use eyre::Result;
use serde::Deserialize;
use serde::Serialize;
use storage::schedule::model::Day;
use teloxide::prelude::Requester as _;
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup, Message};

#[derive(Default)]
pub struct DayView {
    pub date: chrono::NaiveDate,
    pub go_back: Option<Widget>,
}

impl DayView {
    pub fn new(date: chrono::NaiveDate, go_back: Option<Widget>) -> Self {
        Self { date, go_back }
    }
}

#[async_trait]
impl View for DayView {
    async fn show(&mut self, ctx: &mut Context) -> Result<()> {
        let day = ctx.ledger.get_day(self.date).await?;
        let (msg, keymap) = render_day(ctx, &day, self.go_back.is_some());
        ctx.edit_origin(&msg, keymap).await?;
        Ok(())
    }

    async fn handle_message(
        &mut self,
        ctx: &mut Context,
        message: &Message,
    ) -> Result<Option<Widget>> {
        ctx.bot.delete_message(message.chat.id, message.id).await?;
        Ok(None)
    }

    async fn handle_callback(&mut self, ctx: &mut Context, data: &str) -> Result<Option<Widget>> {
        match CalendarDayCallback::from_data(data)? {
            CalendarDayCallback::AddTraining(_) => todo!(),
            CalendarDayCallback::SelectTraining(_) => todo!(),
            CalendarDayCallback::SelectDay(day) => {
                self.date = day;
                self.show(ctx).await?;
            }
            CalendarDayCallback::Back => {
                if let Some(widget) = self.go_back.take() {
                    return Ok(Some(widget));
                }
            }
        }
        Ok(None)
    }
}

fn render_day(ctx: &Context, day: &Day, has_back: bool) -> (String, InlineKeyboardMarkup) {
    let msg = format!(
        "
📅  Расписание на *{}*:
➖➖➖➖➖➖➖➖➖➖➖➖➖➖➖➖➖
🟢 \\- открыта запись 
🟣 \\- мест нет
🟠 \\- запись закрыта
🔵 \\- идет тренировка
⛔ \\- отменено 
✔️  \\- завершено
➖➖➖➖➖➖➖➖➖➖➖➖➖➖➖➖➖
{}
        ",
        day.date.format("%d\\.%m\\.%Y"),
        if day.training.is_empty() {
            "нет занятий 🌴"
        } else {
            ""
        }
    );
    let mut keymap = InlineKeyboardMarkup::default();

    for training in &day.training {
        let mut row = vec![];
        row.push(InlineKeyboardButton::callback(
            format!(
                "{} {} {}",
                render_training_status(&training.status),
                training.start_at.format("%H:%M"),
                training.name.as_str(),
            ),
            format!("slc_training_{}", training.id),
        ));
        keymap = keymap.append_row(row);
    }

    if ctx.has_right(storage::user::rights::Rule::EditSchedule) {
        keymap = keymap.append_row(vec![InlineKeyboardButton::callback(
            "📝  запланировать тренировку",
            CalendarDayCallback::AddTraining(day.date).to_data(),
        )]);
    }

    let mut nav_row = vec![];
    let now = chrono::Local::now().naive_local().date();
    if now < day.date {
        let prev = day.date - Duration::days(1);
        nav_row.push(InlineKeyboardButton::callback(
            format!("{} ⬅️", prev.format("%d.%m")),
            CalendarDayCallback::SelectDay(prev).to_data(),
        ));
    }

    if ctx.ledger.has_week(day.date + Duration::days(1)) {
        let next = day.date + Duration::days(1);
        nav_row.push(InlineKeyboardButton::callback(
            format!("➡️ {}", next.format("%d.%m")),
            CalendarDayCallback::SelectDay(next).to_data(),
        ));
    }
    keymap = keymap.append_row(nav_row);

    if has_back {
        keymap = keymap.append_row(vec![InlineKeyboardButton::callback(
            "📅 Назад",
            CalendarDayCallback::Back.to_data(),
        )]);
    }
    (msg, keymap)
}

#[derive(Debug, Serialize, Deserialize)]
pub enum CalendarDayCallback {
    AddTraining(NaiveDate),
    SelectTraining(u64),
    SelectDay(NaiveDate),
    Back,
}

