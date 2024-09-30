use chrono::{DateTime, Local};
use eyre::Error;
use model::{
    session,
    statistics::{
        calendar::LedgerStatistics,
        history::{SubscriptionStatistics, SubscriptionsStatisticsCollector},
    },
};
use mongodb::bson::oid::ObjectId;

use super::{calendar::Calendar, history::History, users::Users};

#[derive(Clone)]
pub struct Statistics {
    calendar: Calendar,
    history: History,
    users: Users,
}

impl Statistics {
    pub(crate) fn new(calendar: Calendar, history: History, users: Users) -> Self {
        Self {
            calendar,
            history,
            users,
        }
    }

    pub async fn calendar(
        &self,
        session: &mut session::Session,
        from: Option<DateTime<Local>>,
        to: Option<DateTime<Local>>,
    ) -> Result<LedgerStatistics, Error> {
        let mut stat = LedgerStatistics::default();
        let mut cursor = self.calendar.find_range(session, from, to).await?;
        while let Some(day) = cursor.next(&mut *session).await {
            stat.extend(day?);
        }

        Ok(stat)
    }

    pub async fn subscriptions(
        &self,
        session: &mut session::Session,
        from: Option<DateTime<Local>>,
        to: Option<DateTime<Local>>,
    ) -> Result<SubscriptionStatistics, Error> {
        //todo find actual object id
        let mut stat =
            SubscriptionsStatisticsCollector::new(ObjectId::parse_str("66eaa1c8fbb6dfac1e816139")?);
        let mut cursor = self.history.find_range(session, from, to).await?;
        while let Some(day) = cursor.next(&mut *session).await {
            stat.extend(day?);
        }

        for number_to_resolve in stat
            .get_unresolved_presells()
            .into_iter()
            .collect::<Vec<_>>()
        {
            if let Some(user) = self
                .users
                .find_by_phone(session, &number_to_resolve)
                .await?
            {
                stat.resolve_presell(&number_to_resolve, user.id);
            }
        }

        Ok(stat.finish())
    }
}
