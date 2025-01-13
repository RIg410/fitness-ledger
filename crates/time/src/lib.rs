pub use chrono;
#[deprecated]
pub mod range;

use chrono::{DateTime, Datelike as _, Duration, Local, Months, TimeZone as _, Weekday};

#[allow(deprecated)]
pub fn at_midnight(date_time: DateTime<Local>) -> DateTime<Local> {
    date_time.date().and_hms(0, 0, 0)
}

pub fn at_mondays_midnight(date_time: DateTime<Local>) -> DateTime<Local> {
    let date = date_time
        .date_naive()
        .week(Weekday::Mon)
        .first_day()
        .and_hms_opt(0, 0, 0)
        .unwrap();
    Local.from_local_datetime(&date).earliest().unwrap()
}

#[allow(deprecated)]
pub fn at_first_day_of_month(date_time: DateTime<Local>) -> DateTime<Local> {
    date_time.date().with_day(1).unwrap().and_hms(0, 0, 0)
}

pub fn at_last_day_of_month(date_time: DateTime<Local>) -> DateTime<Local> {
    at_first_day_of_month(date_time)
        .checked_add_months(Months::new(1))
        .unwrap()
        - Duration::seconds(1)
}
