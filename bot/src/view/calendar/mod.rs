use std::vec;

use super::{training::schedule_training::ScheduleTraining, View};
use crate::{callback_data::Calldata as _, context::Context, state::Widget};
use async_trait::async_trait;
use chrono::{DateTime, Datelike, Local, TimeZone, Timelike as _, Weekday};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use storage::{calendar::model::Week, training::model::TrainingStatus, user::rights::Rule};
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};
use teloxide::{prelude::Requester as _, types::Message};
use training::TrainingView;

pub mod training;

pub struct CalendarView {
    go_back: Option<Widget>,
    weed_id: DateTime<Local>,
    selected_day: Weekday,
    date: DateTime<Local>,
    filter: Filter,
}

impl CalendarView {
    pub fn new(
        id: DateTime<Local>,
        go_back: Option<Widget>,
        selected_day: Option<Weekday>,
        filter: Option<Filter>,
    ) -> Self {
        Self {
            go_back,
            weed_id: id,
            selected_day: selected_day.unwrap_or(Local::now().weekday()),
            date: id,
            filter: filter.unwrap_or_default(),
        }
    }
}

#[async_trait]
impl View for CalendarView {
    async fn show(&mut self, ctx: &mut Context) -> Result<(), eyre::Error> {
        let week = ctx.ledger.calendar.get_week(Some(self.weed_id)).await?;
        self.date = week.day_date(self.selected_day);
        let (text, keymap) = render_week(
            ctx,
            &week,
            ctx.ledger.calendar.has_prev_week(&week),
            ctx.ledger.calendar.has_next_week(&week),
            self.go_back.is_some(),
            self.selected_day,
            &self.filter,
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
                self.selected_day = day;
                self.show(ctx).await?;
                Ok(None)
            }
            CalendarCallback::Back => {
                if let Some(widget) = self.go_back.take() {
                    Ok(Some(widget))
                } else {
                    Ok(None)
                }
            }
            CalendarCallback::SelectTraining(id) => {
                return Ok(Some(Box::new(TrainingView::new(
                    id.into(),
                    Some(Box::new(CalendarView::new(
                        self.weed_id,
                        self.go_back.take(),
                        Some(self.selected_day),
                        Some(self.filter.clone()),
                    ))),
                ))));
            }
            CalendarCallback::AddTraining => {
                ctx.ensure(Rule::EditSchedule)?;
                let widget = Box::new(CalendarView::new(
                    self.weed_id,
                    self.go_back.take(),
                    Some(self.selected_day),
                    Some(self.filter.clone()),
                ));
                return Ok(Some(Box::new(ScheduleTraining::new(
                    self.date,
                    Some(widget),
                ))));
            }
        }
    }
}

pub fn render_week(
    ctx: &Context,
    week: &Week,
    has_prev: bool,
    hes_next: bool,
    has_back: bool,
    selected_day: Weekday,
    filter: &Filter,
) -> (String, InlineKeyboardMarkup) {
    let msg = format!(
        "
📅  Расписание
*{} {}*
с *{}* по *{}*
➖➖➖➖➖➖➖➖➖➖➖➖➖➖➖➖
🟢\\- запись открыта ⛔\\- тренировка отменена
🟠\\- запись закрыта ✔️\\- тренировка прошла
🔵\\- тренировка идет
➖➖➖➖➖➖➖➖➖➖➖➖➖➖➖➖
",
        month(&week.id),
        week.id.year(),
        week.days().next().unwrap().1.format("%d\\.%m"),
        week.days().last().unwrap().1.format("%d\\.%m"),
    );

    let mut buttons = InlineKeyboardMarkup::default();
    let mut row = vec![];
    for (day, date) in week.days() {
        let text = format!(
            "{}{}",
            if day.weekday == selected_day {
                "🟢"
            } else {
                ""
            },
            render_weekday(&date)
        );
        row.push(InlineKeyboardButton::callback(
            text,
            CalendarCallback::SelectDay(date.weekday()).to_data(),
        ));
    }
    buttons = buttons.append_row(row);
    let mut row = vec![];
    if has_prev {
        row.push(InlineKeyboardButton::callback(
            "⬅️ предыдущая неделя",
            CalendarCallback::GoToWeek(week.prev_week_id().into()).to_data(),
        ));
    }

    if hes_next {
        row.push(InlineKeyboardButton::callback(
            "➡️ cледующая неделя",
            CalendarCallback::GoToWeek(week.next_week_id().into()).to_data(),
        ));
    }
    buttons = buttons.append_row(row);

    let day = week.get_day(selected_day);

    for training in &day.training {
        if let Some(proto_id) = &filter.proto_id {
            if training.proto_id != *proto_id {
                continue;
            }
        }

        let mut row = vec![];
        row.push(InlineKeyboardButton::callback(
            format!(
                "{} {} {}",
                render_training_status(&training.status, training.is_full()),
                training.start_at.format("%H:%M"),
                training.name.as_str(),
            ),
            CalendarCallback::SelectTraining(training.start_at.into()).to_data(),
        ));
        buttons = buttons.append_row(row);
    }
    if ctx.has_right(storage::user::rights::Rule::EditSchedule) {
        buttons = buttons.append_row(vec![InlineKeyboardButton::callback(
            "📝  запланировать тренировку",
            CalendarCallback::AddTraining.to_data(),
        )]);
    }

    if has_back {
        buttons = buttons.append_row(vec![InlineKeyboardButton::callback(
            "Назад",
            CalendarCallback::Back.to_data(),
        )]);
    }

    (msg, buttons)
}

fn month(datetime: &DateTime<Local>) -> &str {
    match datetime.month() {
        1 => "Январь",
        2 => "Февраль",
        3 => "Март",
        4 => "Апрель",
        5 => "Май",
        6 => "Июнь",
        7 => "Июль",
        8 => "Август",
        9 => "Сентябрь",
        10 => "Октябрь",
        11 => "Ноябрь",
        12 => "Декабрь",
        _ => unreachable!(),
    }
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
    SelectDay(Weekday),
    SelectTraining(CallbackDateTime),
    AddTraining,
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

#[derive(Debug, Default, Clone)]
pub struct Filter {
    pub proto_id: Option<ObjectId>,
}
