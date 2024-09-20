use async_trait::async_trait;
use bot_core::callback_data::{CallbackDateTime, Calldata};
use bot_core::calldata;
use bot_core::context::Context;
use bot_core::widget::{Jmp, View};
use bot_viewer::day::{fmt_dm, fmt_month, fmt_weekday};
use bot_viewer::training::fmt_training_status;
use bot_views::Filter;
use chrono::{DateTime, Datelike, Duration, Local, TimeZone, Timelike as _, Weekday};
use eyre::Error;
use model::ids::{DayId, WeekId};
use model::rights::Rule;
use model::training::TrainingStatus;
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use std::vec;
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};
use teloxide::{prelude::Requester as _, types::Message};
use training::TrainingView;

mod add_client;
mod client;
mod client_list;
pub mod training;

pub struct CalendarView {
    week_id: WeekId,
    selected_day: DayId,
    filter: Filter,
}

impl Default for CalendarView {
    fn default() -> Self {
        Self {
            week_id: WeekId::default(),
            selected_day: Default::default(),
            filter: Default::default(),
        }
    }
}

impl CalendarView {
    pub fn new(week_id: WeekId, selected_day: Option<Weekday>, filter: Option<Filter>) -> Self {
        Self {
            week_id,
            selected_day: week_id.day(selected_day.unwrap_or_else(|| Local::now().weekday())),
            filter: filter.unwrap_or_default(),
        }
    }
}

#[async_trait]
impl View for CalendarView {
    fn name(&self) -> &'static str {
        "CalendarView"
    }

    async fn show(&mut self, ctx: &mut Context) -> Result<(), eyre::Error> {
        let (text, keymap) = render_week(
            ctx,
            self.week_id,
            self.week_id.prev().has_week(),
            self.week_id.next().has_week(),
            self.selected_day,
            &self.filter,
        )
        .await?;
        ctx.edit_origin(&text, keymap).await?;
        Ok(())
    }

    async fn handle_callback(&mut self, ctx: &mut Context, data: &str) -> Result<Jmp, eyre::Error> {
        match calldata!(data) {
            Callback::GoToWeek(week) => {
                self.week_id = WeekId::from(week);
                self.selected_day = self.week_id.day(self.selected_day.week_day());
                Ok(Jmp::None)
            }
            Callback::SelectDay(day) => {
                self.selected_day = DayId::from(day);
                Ok(Jmp::None)
            }
            Callback::SelectTraining(id) => {
                return Ok(TrainingView::new(id.into()).into());
            }
            Callback::AddTraining => {
                ctx.ensure(Rule::EditSchedule)?;
                // return Ok(Some(
                //     ScheduleTraining::new(self.selected_day.local()).boxed(),
                // ));
                todo!()
            }
            Callback::MyTrainings => return todo!(), //Ok(Some(ClientTrainings::new(ctx.me.id).boxed())),
        }
    }
}

pub async fn render_week(
    ctx: &mut Context,
    week_id: WeekId,
    has_prev: bool,
    hes_next: bool,
    selected_day_id: DayId,
    filter: &Filter,
) -> Result<(String, InlineKeyboardMarkup), Error> {
    let week_local = week_id.local();
    let msg = format!(
        "
📅  Расписание
*{} {}*
с *{}* по *{}*
➖➖➖➖➖➖➖➖➖➖➖➖➖➖
🟢\\- запись открыта 
⛔\\- тренировка отменена
🟠\\- запись закрыта 
✔️\\- тренировка прошла
🔵\\- тренировка идет 
❤️\\- моя тренировка
➖➖➖➖➖➖➖➖➖➖➖➖➖➖
",
        fmt_month(&week_local),
        week_local.year(),
        fmt_dm(&week_local),
        fmt_dm(&(week_local + Duration::days(6))),
    );

    let now = Local::now();
    let selected_week_day = selected_day_id.week_day();
    let mut buttons = InlineKeyboardMarkup::default();
    let mut row = vec![];
    for week_day in week() {
        let date = week_id.day(week_day).local();
        let text = format!(
            "{}{}",
            if selected_week_day == week_day {
                "🟢"
            } else {
                ""
            },
            fmt_weekday(&date)
        );
        row.push(InlineKeyboardButton::callback(
            text,
            Callback::SelectDay(date.into()).to_data(),
        ));
    }
    buttons = buttons.append_row(row);
    let mut row = vec![];
    if has_prev {
        row.push(Callback::GoToWeek(week_id.prev().local().into()).button("⬅️ предыдущая неделя"));
    }

    if hes_next {
        row.push(Callback::GoToWeek(week_id.next().local().into()).button("➡️ cледующая неделя"));
    }
    buttons = buttons.append_row(row);
    let mut day = ctx
        .ledger
        .calendar
        .get_day(&mut ctx.session, selected_day_id)
        .await?;
    day.training
        .sort_by(|a, b| a.get_slot().start_at().cmp(&b.get_slot().start_at()));
    for training in &day.training {
        if let Some(proto_id) = &filter.proto_id {
            if training.proto_id != *proto_id {
                continue;
            }
        }

        let start_at = training.get_slot().start_at();
        let mut row = vec![];
        row.push(InlineKeyboardButton::callback(
            format!(
                "{} {} {}",
                fmt_training_status(
                    training.status(now),
                    training.is_processed,
                    training.is_full(),
                    training.clients.contains(&ctx.me.id)
                ),
                start_at.format("%H:%M"),
                training.name.as_str(),
            ),
            Callback::SelectTraining(start_at.into()).to_data(),
        ));
        buttons = buttons.append_row(row);
    }

    buttons = buttons.append_row(Callback::MyTrainings.btn_row("🫶🏻 Мои тренировки"));

    if ctx.has_right(Rule::EditSchedule) {
        buttons = buttons.append_row(Callback::AddTraining.btn_row("📝  запланировать тренировку"));
    }

    Ok((msg, buttons))
}

fn week() -> [Weekday; 7] {
    [
        Weekday::Mon,
        Weekday::Tue,
        Weekday::Wed,
        Weekday::Thu,
        Weekday::Fri,
        Weekday::Sat,
        Weekday::Sun,
    ]
}

#[derive(Debug, Serialize, Deserialize)]
enum Callback {
    GoToWeek(CallbackDateTime),
    SelectDay(CallbackDateTime),
    SelectTraining(CallbackDateTime),
    AddTraining,
    MyTrainings,
}
