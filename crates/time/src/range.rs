use chrono::{DateTime, Datelike as _, Duration, Local, Months, Timelike as _};

#[derive(Clone, Copy)]
pub enum Range {
    Full,
    Month(DateTime<Local>),
    Range(Option<DateTime<Local>>, Option<DateTime<Local>>),
}

impl Range {
    pub fn range(&self) -> (Option<DateTime<Local>>, Option<DateTime<Local>>) {
        match self {
            Range::Full => (None, None),
            Range::Month(date_time) => {
                let from = date_time
                    .with_day0(0)
                    .and_then(|dt| dt.with_hour(0))
                    .and_then(|dt| dt.with_minute(0))
                    .and_then(|dt| dt.with_second(0));

                let to = from
                    .and_then(|dt| dt.checked_add_months(Months::new(1)))
                    .map(|dt| dt - Duration::seconds(1));
                (from, to)
            }
            Range::Range(from, to) => (*from, *to),
        }
    }

    pub fn is_month(&self) -> bool {
        matches!(self, Range::Month(_))
    }

    pub fn next_month(&self) -> Self {
        match self {
            Range::Full => Range::Month(Local::now()),
            Range::Month(date) => Range::Month(date.checked_add_months(Months::new(1)).unwrap()),
            Range::Range(_, _) => Range::Full,
        }
    }

    pub fn prev_month(&self) -> Self {
        match self {
            Range::Full => Range::Month(Local::now()),
            Range::Month(date) => Range::Month(date.checked_sub_months(Months::new(1)).unwrap()),
            Range::Range(_, _) => Range::Full,
        }
    }
}
