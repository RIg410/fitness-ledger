use chrono::{Datelike, NaiveDate, Weekday};
use itertools::Itertools;
use storage::{
    schedule::model::{Day, Week},
    training::model::TrainingStatus,
};
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};

use super::ScheduleCalendarCallback;

pub fn render_week(week: &Week, has_prev: bool, hes_next: bool) -> (String, InlineKeyboardMarkup) {
    let mut msg = format!(
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
    for day in &week.days {
        let now = chrono::Local::now().naive_local().date();
        if day.date < now {
            continue;
        }

        let mut row = vec![];
        let short = day.training.iter().map(|t| &t.short_name).join("/");

        let name = format!(
            "{} {} : {} {}                                 ",
            render_weekday(&day.date),
            day.date.format("%d.%m"),
            render_day_status(&day),
            short
        );

        row.push(InlineKeyboardButton::callback(
            name,
            ScheduleCalendarCallback::SelectDay(day.date).to_data(),
        ));
        buttons = buttons.append_row(row);
    }

    let mut last_row = vec![];
    if has_prev {
        last_row.push(InlineKeyboardButton::callback(
            "⬅️",
            ScheduleCalendarCallback::GoToWeek(week.prev_week_id()).to_data(),
        ));
    }

    if hes_next {
        last_row.push(InlineKeyboardButton::callback(
            "➡️",
            ScheduleCalendarCallback::GoToWeek(week.next_week_id()).to_data(),
        ));
    }
    buttons = buttons.append_row(last_row);
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
