use super::{render_training_status, View};
use crate::{context::Context, state::Widget};
use async_trait::async_trait;
use chrono::{Duration, NaiveDate};
use eyre::eyre;
use eyre::Result;
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
        match CalendarDayCallback::try_from(data)? {
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
        nav_row.push(InlineKeyboardButton::callback(
            "⬅️",
            CalendarDayCallback::SelectDay(day.date - Duration::days(1)).to_data(),
        ));
    }

    if ctx.ledger.has_week(day.date + Duration::days(1)) {
        nav_row.push(InlineKeyboardButton::callback(
            "➡️",
            CalendarDayCallback::SelectDay(day.date + Duration::days(1)).to_data(),
        ));
    }
    keymap = keymap.append_row(nav_row);

    if has_back {
        keymap = keymap.append_row(vec![InlineKeyboardButton::callback(
            "🔙 Назад",
            CalendarDayCallback::Back.to_data(),
        )]);
    }
    (msg, keymap)
}

#[derive(Clone, Debug)]
pub enum CalendarDayCallback {
    AddTraining(NaiveDate),
    SelectTraining(u64),
    SelectDay(NaiveDate),
    Back,
}

impl CalendarDayCallback {
    pub fn to_data(&self) -> String {
        match self {
            CalendarDayCallback::AddTraining(date) => {
                format!("cdc_add_training:{}", date.format("%Y-%m-%d"))
            }
            CalendarDayCallback::SelectTraining(id) => format!("cdc_select_training:{}", id),
            CalendarDayCallback::SelectDay(date) => {
                format!("cdc_select_day:{}", date.format("%Y-%m-%d"))
            }
            CalendarDayCallback::Back => "cdc_back:".to_string(),
        }
    }
}

impl TryFrom<&str> for CalendarDayCallback {
    type Error = eyre::Error;

    fn try_from(value: &str) -> std::result::Result<Self, Self::Error> {
        let parts: Vec<&str> = value.split(':').collect();
        if parts.len() != 2 {
            return Err(eyre!("Invalid CalendarDayCallback"));
        }

        let date = NaiveDate::parse_from_str(parts[1], "%Y-%m-%d")?;
        match parts[0] {
            "cdc_add_training" => Ok(Self::AddTraining(date)),
            "cdc_select_training" => Ok(Self::SelectTraining(parts[1].parse()?)),
            "cdc_select_day" => Ok(Self::SelectDay(date)),
            "cdc_back" => Ok(Self::Back),
            _ => Err(eyre!("Invalid CalendarDayCallback")),
        }
    }
}
