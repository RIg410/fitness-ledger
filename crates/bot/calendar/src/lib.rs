use async_trait::async_trait;
use bot_core::callback_data::{CallbackDateTime, Calldata, TrainingIdCallback};
use bot_core::calldata;
use bot_core::context::Context;
use bot_core::widget::{Jmp, View};
use bot_trainigs::list::TrainingList;
use bot_trainigs::program::list::ProgramList;
use bot_trainigs::schedule::ScheduleTrainingPreset;
use bot_trainigs::view::TrainingView;
use bot_viewer::day::{fmt_dm, fmt_month, fmt_weekday};
use bot_viewer::rooms::fmt_room_emoji;
use bot_viewer::training::fmt_training_status;
use bot_views::Filter;
use chrono::{Datelike, Duration, Local, Weekday};
use eyre::Error;
use model::ids::{DayId, WeekId};
use model::rights::Rule;
use model::rooms::Room;
use model::training::Statistics;
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use std::vec;
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};
use teloxide::utils::markdown::escape;

#[derive(Default)]
pub struct CalendarView {
    week_id: WeekId,
    selected_day: DayId,
    filter: Filter,
    rooms: Vec<ObjectId>,
}

impl CalendarView {
    pub fn new(week_id: WeekId, selected_day: Option<Weekday>, filter: Option<Filter>) -> Self {
        Self {
            week_id,
            selected_day: week_id.day(selected_day.unwrap_or_else(|| Local::now().weekday())),
            filter: filter.unwrap_or_default(),
            rooms: vec![],
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
            self.week_id.prev().has_week() || ctx.is_employee() || ctx.is_admin(),
            self.week_id.next().has_week() || ctx.is_employee() || ctx.is_admin(),
            self.selected_day,
            &self.filter,
            &self.rooms,
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
                Ok(Jmp::Stay)
            }
            Callback::SelectDay(day) => {
                self.selected_day = DayId::from(day);
                Ok(Jmp::Stay)
            }
            Callback::SelectTraining(id) => Ok(TrainingView::new(id.into()).into()),
            Callback::AddTraining => {
                ctx.ensure(Rule::EditSchedule)?;
                let preset = ScheduleTrainingPreset::with_day(self.selected_day.local());
                Ok(ProgramList::new(preset).into())
            }
            Callback::MyTrainings => {
                if ctx.me.employee.is_some() {
                    Ok(TrainingList::couches(ctx.me.id).into())
                } else {
                    Ok(TrainingList::users(ctx.me.id).into())
                }
            }
            Callback::SelectRoom(room) => {
                let room_id = ObjectId::from_bytes(room);
                if self.rooms.contains(&room_id) {
                    self.rooms.retain(|r| r != &room_id);
                } else {
                    self.rooms.push(room_id);
                }
                Ok(Jmp::Stay)
            }
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
    rooms: &[ObjectId],
) -> Result<(String, InlineKeyboardMarkup), Error> {
    let week_local = week_id.local();
    let selected_week_day = selected_day_id.week_day();

    let mut msg = format!(
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
Выбранный день:*{}*
",
        fmt_month(&week_local),
        week_local.year(),
        fmt_dm(&week_local),
        fmt_dm(&(week_local + Duration::days(6))),
        fmt_dm(&week_id.day(selected_week_day).local())
    );

    let now = Local::now();
    let mut buttons = InlineKeyboardMarkup::default();
    let adult_room_name = if rooms.contains(&Room::Adult.id()) {
        "✅🧘 Взрослые"
    } else {
        "🧘 Взрослые"
    };

    let child_room_name = if rooms.contains(&Room::Child.id()) {
        "✅🧒 Дети"
    } else {
        "🧒 Дети"
    };

    buttons = buttons.append_row(vec![
        Callback::SelectRoom(Room::Adult.id().bytes()).button(adult_room_name),
        Callback::SelectRoom(Room::Child.id().bytes()).button(child_room_name),
    ]);

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
            fmt_weekday(date.weekday())
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

    if ctx.has_right(Rule::ViewFinance) {
        let stat = day
            .training
            .iter()
            .filter_map(|t| t.statistics)
            .sum::<Statistics>();
        msg.push_str(&escape(&format!(
            "\n📊Заработано {}\n📊Награда инструктора {}",
            stat.earned, stat.couch_rewards
        )));
    }

    day.training.sort_by_key(|a| a.get_slot().start_at());
    for training in &day.training {
        if let Some(proto_id) = &filter.proto_id {
            if training.proto_id != *proto_id {
                continue;
            }
        }

        if !rooms.is_empty() && !rooms.contains(&training.room()) {
            continue;
        }

        let start_at = training.get_slot().start_at();
        let mut row = vec![];
        row.push(InlineKeyboardButton::callback(
            format!(
                "{} {} {}{}",
                fmt_training_status(
                    training.status(now),
                    training.is_processed,
                    training.is_full(),
                    training.clients.contains(&ctx.me.id)
                ),
                start_at.format("%H:%M"),
                fmt_room_emoji(Room::from(training.room())),
                training.name.as_str(),
            ),
            Callback::SelectTraining(training.id().into()).to_data(),
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
    SelectTraining(TrainingIdCallback),
    AddTraining,
    MyTrainings,
    SelectRoom([u8; 12]),
}
