use std::vec;

use super::{training::schedule_training::ScheduleTraining, View};
use crate::{callback_data::Calldata as _, context::Context, state::Widget};
use async_trait::async_trait;
use chrono::{DateTime, Datelike, Duration, Local, TimeZone, Timelike as _, Weekday};
use eyre::Error;
use model::ids::{DayId, WeekId};
use model::rights::Rule;
use model::training::TrainingStatus;
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};
use teloxide::{prelude::Requester as _, types::Message};
use training::TrainingView;
pub mod training;

pub struct CalendarView {
    go_back: Option<Widget>,
    week_id: WeekId,
    selected_day: DayId,
    filter: Filter,
}

impl CalendarView {
    pub fn new(
        week_id: WeekId,
        go_back: Option<Widget>,
        selected_day: Option<Weekday>,
        filter: Option<Filter>,
    ) -> Self {
        Self {
            go_back,
            week_id,
            selected_day: week_id.day(selected_day.unwrap_or_else(|| Local::now().weekday())),
            filter: filter.unwrap_or_default(),
        }
    }
}

#[async_trait]
impl View for CalendarView {
    async fn show(&mut self, ctx: &mut Context) -> Result<(), eyre::Error> {
        let (text, keymap) = render_week(
            ctx,
            self.week_id,
            self.week_id.prev().has_week(),
            self.week_id.next().has_week(),
            self.go_back.is_some(),
            self.selected_day,
            &self.filter,
        )
        .await?;
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
                self.week_id = WeekId::from(week);
                self.selected_day = self.week_id.day(self.selected_day.week_day());
                self.show(ctx).await?;
                Ok(None)
            }
            CalendarCallback::SelectDay(day) => {
                self.selected_day = DayId::from(day);
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
                        self.week_id,
                        self.go_back.take(),
                        Some(self.selected_day.local().weekday()),
                        Some(self.filter.clone()),
                    ))),
                ))));
            }
            CalendarCallback::AddTraining => {
                ctx.ensure(Rule::EditSchedule)?;
                let widget = Box::new(CalendarView::new(
                    self.week_id,
                    self.go_back.take(),
                    Some(self.selected_day.local().weekday()),
                    Some(self.filter.clone()),
                ));
                return Ok(Some(Box::new(ScheduleTraining::new(
                    self.selected_day.local(),
                    Some(widget),
                ))));
            }
        }
    }
}

pub async fn render_week(
    ctx: &mut Context,
    week_id: WeekId,
    has_prev: bool,
    hes_next: bool,
    has_back: bool,
    selected_day_id: DayId,
    filter: &Filter,
) -> Result<(String, InlineKeyboardMarkup), Error> {
    let week_local = week_id.local();
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
        month(&week_local),
        week_local.year(),
        week_local.format("%d\\.%m"),
        (week_local + Duration::days(6)).format("%d\\.%m"),
    );

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
            render_weekday(&date)
        );
        row.push(InlineKeyboardButton::callback(
            text,
            CalendarCallback::SelectDay(date.into()).to_data(),
        ));
    }
    buttons = buttons.append_row(row);
    let mut row = vec![];
    if has_prev {
        row.push(InlineKeyboardButton::callback(
            "⬅️ предыдущая неделя",
            CalendarCallback::GoToWeek(week_id.prev().local().into()).to_data(),
        ));
    }

    if hes_next {
        row.push(InlineKeyboardButton::callback(
            "➡️ cледующая неделя",
            CalendarCallback::GoToWeek(week_id.next().local().into()).to_data(),
        ));
    }
    buttons = buttons.append_row(row);
    let day = ctx
        .ledger
        .calendar
        .get_day(&mut ctx.session, selected_day_id)
        .await?;
    for training in &day.training {
        if let Some(proto_id) = &filter.proto_id {
            if training.proto_id != *proto_id {
                continue;
            }
        }

        let start_at = training.start_at_local();
        let mut row = vec![];
        row.push(InlineKeyboardButton::callback(
            format!(
                "{} {} {}",
                render_training_status(&training.status, training.is_full()),
                start_at.format("%H:%M"),
                training.name.as_str(),
            ),
            CalendarCallback::SelectTraining(start_at.into()).to_data(),
        ));
        buttons = buttons.append_row(row);
    }
    if ctx.has_right(Rule::EditSchedule) {
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

    Ok((msg, buttons))
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

pub fn render_weekday(weekday: &DateTime<Local>) -> &'static str {
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
    SelectDay(CallbackDateTime),
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

impl From<CallbackDateTime> for WeekId {
    fn from(date: CallbackDateTime) -> Self {
        let local = DateTime::<Local>::from(date);
        WeekId::new(local)
    }
}

impl From<CallbackDateTime> for DayId {
    fn from(date: CallbackDateTime) -> Self {
        let local = DateTime::<Local>::from(date);
        DayId::from(local)
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
