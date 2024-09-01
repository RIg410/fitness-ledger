use super::training::TrainingView;
use super::{render_training_status, CallbackDateTime, View};
use crate::callback_data::Calldata as _;
use crate::view::training::schedule_training::ScheduleTraining;
use crate::{context::Context, state::Widget};
use async_trait::async_trait;
use chrono::{DateTime, Duration, Local};
use eyre::Result;
use serde::Deserialize;
use serde::Serialize;
use storage::calendar::day_id;
use storage::calendar::model::Day;
use storage::user::rights::Rule;
use teloxide::prelude::Requester as _;
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup, Message};

#[derive(Default)]
pub struct DayView {
    pub date: DateTime<Local>,
    pub go_back: Option<Widget>,
}

impl DayView {
    pub fn new(date: DateTime<Local>, go_back: Option<Widget>) -> Self {
        Self { date, go_back }
    }
}

#[async_trait]
impl View for DayView {
    async fn show(&mut self, ctx: &mut Context) -> Result<()> {
        let day = ctx.ledger.calendar.get_day(self.date).await?;
        let (msg, keymap) = render_day(ctx, &day, self.date, self.go_back.is_some());
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
            CalendarDayCallback::AddTraining => {
                ctx.ensure(Rule::EditSchedule)?;

                let widget = Box::new(DayView::new(self.date, self.go_back.take()));
                return Ok(Some(Box::new(ScheduleTraining::new(
                    self.date,
                    Some(widget),
                ))));
            }
            CalendarDayCallback::SelectTraining(id) => {
                return Ok(Some(Box::new(TrainingView::new(
                    id.into(),
                    Some(Box::new(DayView::new(self.date, self.go_back.take()))),
                ))));
            }
            CalendarDayCallback::Next => {
                self.date += Duration::days(1);
                self.show(ctx).await?;
            }
            CalendarDayCallback::Prev => {
                self.date -= Duration::days(1);
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

fn render_day(
    ctx: &Context,
    day: &Day,
    date: DateTime<Local>,
    has_back: bool,
) -> (String, InlineKeyboardMarkup) {
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
        date.format("%d\\.%m\\.%Y"),
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
                render_training_status(&training.status, training.is_full()),
                training.start_at.format("%H:%M"),
                training.name.as_str(),
            ),
            CalendarDayCallback::SelectTraining(training.start_at.into()).to_data(),
        ));
        keymap = keymap.append_row(row);
    }

    if ctx.has_right(storage::user::rights::Rule::EditSchedule) {
        keymap = keymap.append_row(vec![InlineKeyboardButton::callback(
            "📝  запланировать тренировку",
            CalendarDayCallback::AddTraining.to_data(),
        )]);
    }

    let mut nav_row = vec![];
    let now = day_id(chrono::Local::now()).unwrap_or_else(|| chrono::Local::now());
    if now < date {
        let prev = date - Duration::days(1);
        nav_row.push(InlineKeyboardButton::callback(
            format!("{} ⬅️", prev.format("%d.%m")),
            CalendarDayCallback::Prev.to_data(),
        ));
    }

    if ctx.ledger.calendar.has_week(date + Duration::days(1)) {
        let next = date + Duration::days(1);
        nav_row.push(InlineKeyboardButton::callback(
            format!("➡️ {}", next.format("%d.%m")),
            CalendarDayCallback::Next.to_data(),
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
    AddTraining,
    SelectTraining(CallbackDateTime),
    Next,
    Prev,
    Back,
}
