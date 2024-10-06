use super::{calendar::Calendar, history::History, users::Users};
use chrono::{DateTime, Local};
use eyre::Error;
use model::{
    session,
    statistics::{
        calendar::LedgerStatistics,
        history::{SubscriptionStatistics, SubscriptionsStatisticsCollector},
        marketing::UsersStat,
    },
};
use mongodb::bson::oid::ObjectId;

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

        let come_from = self.users_statistic(session, from, to).await?;

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

        Ok(stat.finish(come_from))
    }

    async fn users_statistic(
        &self,
        session: &mut session::Session,
        from: Option<DateTime<Local>>,
        to: Option<DateTime<Local>>,
    ) -> Result<UsersStat, Error> {
        let mut stat = UsersStat::default();
        let mut cursor = self.users.find_all(session, from, to).await?;
        while let Some(user) = cursor.next(&mut *session).await {
            stat.extend(&user?);
        }

        Ok(stat)
    }
}
