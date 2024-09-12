use chrono::{DateTime, Datelike as _, Local, TimeZone as _, Utc, Weekday};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct WeekId(DateTime<Utc>);

impl WeekId {
    pub fn new(date_time: DateTime<Local>) -> Self {
        let date = date_time
            .date_naive()
            .week(Weekday::Mon)
            .first_day()
            .and_hms_opt(0, 0, 0)
            .unwrap();
        let local_date = Local.from_local_datetime(&date).earliest().unwrap();
        WeekId(local_date.with_timezone(&Utc))
    }

    pub fn local(&self) -> DateTime<Local> {
        self.0.with_timezone(&Local)
    }

    pub fn id(&self) -> DateTime<Utc> {
        self.0
    }

    pub fn next(&self) -> Self {
        WeekId(self.0 + chrono::Duration::days(7))
    }

    pub fn prev(&self) -> Self {
        WeekId(self.0 - chrono::Duration::days(7))
    }

    pub fn has_week(&self) -> bool {
        let now = Utc::now();
        let max_year = now.year() + 2;
        let current_year = self.0.year();
        current_year <= max_year && self.next().0 > now
    }

    pub fn day(&self, weekday: Weekday) -> DayId {
        let date = self.local() + chrono::Duration::days(weekday.num_days_from_monday() as i64);
        DayId(date.with_timezone(&Utc))
    }
}

impl Default for WeekId {
    fn default() -> Self {
        WeekId::new(Local::now())
    }
}

impl From<DateTime<Local>> for WeekId {
    fn from(date_time: DateTime<Local>) -> Self {
        WeekId::new(date_time)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct DayId(DateTime<Utc>);

impl DayId {
    pub fn new(date_time: DateTime<Local>) -> Self {
        let date = date_time.date().and_hms(0, 0, 0);
        DayId(date.with_timezone(&Utc))
    }

    /// Create DayId from Utc DateTime
    /// # Safety 
    pub unsafe fn from_utc(date_time: DateTime<Utc>) -> Self {
        DayId(date_time)
    }

    pub fn local(&self) -> DateTime<Local> {
        self.0.with_timezone(&Local)
    }

    pub fn id(&self) -> DateTime<Utc> {
        self.0
    }

    pub fn week_day(&self) -> Weekday {
        self.local().weekday()
    }

    pub fn week_id(&self) -> WeekId {
        WeekId::new(self.local())
    }

    pub fn next(&self) -> Self {
        DayId(self.0 + chrono::Duration::days(1))
    }

    pub fn prev(&self) -> Self {
        DayId(self.0 - chrono::Duration::days(1))
    }
}

impl From<DateTime<Local>> for DayId {
    fn from(date_time: DateTime<Local>) -> Self {
        DayId::new(date_time)
    }
}

impl From<DateTime<Utc>> for DayId {
    fn from(date_time: DateTime<Utc>) -> Self {
        DayId::from(date_time.with_timezone(&Local))
    }
}

impl Default for DayId {
    fn default() -> Self {
        DayId::new(Local::now())
    }
}
