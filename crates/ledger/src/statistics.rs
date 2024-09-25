use crate::calendar::Calendar;
use chrono::{DateTime, Local};
use eyre::Error;
use model::{session, statistics::LedgerStatistics};

#[derive(Clone)]
pub struct Statistics {
    calendar: Calendar,
}

impl Statistics {
    pub(crate) fn new(calendar: Calendar) -> Self {
        Self { calendar }
    }

    pub async fn calculate(
        &self,
        session: &mut session::Session,
        from: DateTime<Local>,
        to: DateTime<Local>,
    ) -> Result<LedgerStatistics, Error> {
        let mut stat = LedgerStatistics::default();
        let mut cursor = self.calendar.find_range(session, from, to).await?;
        while let Some(day) = cursor.next(&mut *session).await {
            stat.extend(day?);
        }

        Ok(stat)
    }
}
