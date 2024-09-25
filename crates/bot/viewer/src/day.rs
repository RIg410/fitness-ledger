use chrono::{
    format::{DelayedFormat, StrftimeItems},
    DateTime, Datelike as _, Local, Weekday,
};

pub fn fmt_weekday(day: Weekday) -> &'static str {
    match day {
        Weekday::Mon => "Пн",
        Weekday::Tue => "Вт",
        Weekday::Wed => "Ср",
        Weekday::Thu => "Чт",
        Weekday::Fri => "Пт",
        Weekday::Sat => "Сб",
        Weekday::Sun => "Вс",
    }
}

pub fn fmt_date(day: &DateTime<Local>) -> DelayedFormat<StrftimeItems> {
    day.format("%d\\.%m\\.%Y")
}

pub fn fmt_dm(day: &DateTime<Local>) -> DelayedFormat<StrftimeItems> {
    day.format("%d\\.%m")
}

pub fn fmt_dt(day: &DateTime<Local>) -> DelayedFormat<StrftimeItems> {
    day.format("%d\\.%m\\.%Y %H:%M")
}

pub fn fmt_month(datetime: &DateTime<Local>) -> &str {
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
