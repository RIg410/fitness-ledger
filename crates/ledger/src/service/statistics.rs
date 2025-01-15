use super::{calendar::Calendar, history::History, requests::Requests, users::Users};

pub struct Statistics {
    calendar: Calendar,
    history: History,
    users: Users,
    requests: Requests,
}

impl Statistics {
    pub(crate) fn new(
        calendar: Calendar,
        history: History,
        users: Users,
        requests: Requests,
    ) -> Self {
        Self {
            calendar,
            history,
            users,
            requests,
        }
    }

    // pub async fn calendar(
    //     &self,
    //     session: &mut session::Session,
    //     from: Option<DateTime<Local>>,
    //     to: Option<DateTime<Local>>,
    // ) -> Result<LedgerStatistics, Error> {
    //     let mut stat = LedgerStatistics::default();
    //     let mut cursor = self.calendar.find_range(session, from, to).await?;
    //     while let Some(day) = cursor.next(&mut *session).await {
    //         stat.extend(day?);
    //     }

    //     Ok(stat)
    // }

    // pub async fn subscriptions(
    //     &self,
    //     session: &mut session::Session,
    //     from: Option<DateTime<Local>>,
    //     to: Option<DateTime<Local>>,
    // ) -> Result<SubscriptionStatistics, Error> {
    //     //todo find actual object id
    //     let mut stat =
    //         SubscriptionsStatisticsCollector::new(ObjectId::parse_str("66eaa1c8fbb6dfac1e816139")?);
    //     let mut cursor = self.history.find_range(session, from, to).await?;
    //     while let Some(day) = cursor.next(&mut *session).await {
    //         stat.extend(day?);
    //     }

    //     let mut requests_cursor = self.requests.cursor(session, from, to).await?;

    //     let mut come_from_map = HashMap::new();
    //     let mut requests_map = HashMap::new();
    //     while let Some(request) = requests_cursor.next(&mut *session).await {
    //         let request = request?;

    //         let req_counter = requests_map.entry(request.come_from).or_default();
    //         *req_counter += 1;

    //         come_from_map.insert(sanitize_phone(&request.phone), request.come_from);
    //     }

    //     let come_from = self
    //         .users_statistic(session, from, to, &come_from_map)
    //         .await
    //         .context("Failed to gather user statistics")?;

    //     for number_to_resolve in stat
    //         .get_unresolved_presells()
    //         .into_iter()
    //         .collect::<Vec<_>>()
    //     {
    //         if let Some(user) = self
    //             .users
    //             .find_by_phone(session, &number_to_resolve)
    //             .await?
    //         {
    //             stat.resolve_presell(&number_to_resolve, user.id);
    //         }
    //     }

    //     Ok(stat.finish(come_from, requests_map))
    // }

    // async fn users_statistic(
    //     &self,
    //     session: &mut session::Session,
    //     from: Option<DateTime<Local>>,
    //     to: Option<DateTime<Local>>,
    //     come_from_map: &HashMap<String, ComeFrom>,
    // ) -> Result<UsersStat, Error> {
    //     let mut stat = UsersStat::default();
    //     let mut cursor = self.users.find_all(session, from, to).await?;
    //     while let Some(user) = cursor.next(&mut *session).await {
    //         let mut user = user?;
    //         if let Some(phone) = user.phone.as_ref() {
    //             if let Some(come_from) = come_from_map.get(&sanitize_phone(phone)) {
    //                 user.come_from = *come_from;
    //             }
    //         }
    //         stat.extend(&user);
    //     }

    //     Ok(stat)
    // }
}
