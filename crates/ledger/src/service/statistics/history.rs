use crate::service::{history::History, requests::Requests, users::Users};
use chrono::NaiveDate;
use eyre::Result;
use model::{
    history::Action,
    session::Session,
    statistics::month::{MonthStatistics, SourceStat, SubscriptionStat},
};
use std::collections::HashMap;

use super::aggregation::month_range;

pub async fn load_requests_and_history(
    session: &mut Session,
    month_id: NaiveDate,
    requests: &Requests,
    history: &History,
    users: &Users,
    month: &mut MonthStatistics,
) -> Result<()> {
    let (start, end) = month_range(&month_id);

    let mut user_for_marketing = HashMap::new();
    let mut requests = requests.find_range(session, Some(start), Some(end)).await?;
    while let Some(req) = requests.next(session).await {
        let req = req?;

        let stat = month
            .marketing
            .source
            .entry(req.come_from)
            .or_insert_with(|| SourceStat {
                buy_test: 0,
                buy_subscription: 0,
                requests_count: 0,
                earned: 0,
                spent: 0,
            });
        stat.requests_count += 1;

        if let Some(user) = users.find_by_phone(session, &req.phone).await? {
            user_for_marketing.insert(user.id, (req.come_from, 0));
        }
    }

    let mut history = history.find_range(session, Some(start), Some(end)).await?;
    while let Some(row) = history.next(session).await {
        let row = row?;
        match row.action {
            Action::BuySub {
                subscription,
                discount,
            }
            | Action::SellSub {
                subscription,
                discount,
            } => {
                if let Some(user) = user_for_marketing.get_mut(row.sub_actors.first().unwrap()) {
                    let stat = month
                        .marketing
                        .source
                        .entry(user.0)
                        .or_insert_with(|| SourceStat {
                            buy_test: 0,
                            buy_subscription: 0,
                            requests_count: 0,
                            earned: 0,
                            spent: 0,
                        });
                    if user.1 == 0 {
                        stat.buy_test += 1;
                    } else {
                        stat.buy_subscription += 1;
                    }
                    user.1 += 1;
                    stat.earned += (subscription.price
                        - subscription.price * discount.unwrap_or_default())
                    .int_part();
                }

                if let Some(sub) = month
                    .subscriptions
                    .iter_mut()
                    .find(|s| s.name == subscription.name)
                {
                    sub.count += 1;
                    sub.earned = (subscription.price
                        - subscription.price * discount.unwrap_or_default())
                    .int_part();
                    sub.discount = (subscription.price * discount.unwrap_or_default()).int_part();
                } else {
                    month.subscriptions.push(SubscriptionStat {
                        name: subscription.name,
                        count: 1,
                        earned: (subscription.price
                            - subscription.price * discount.unwrap_or_default())
                        .int_part(),
                        burned_training: 0,
                        discount: (subscription.price * discount.unwrap_or_default()).int_part(),
                    });
                }
            }
            Action::ExpireSubscription { subscription } => {
                if let Some(sub) = month
                    .subscriptions
                    .iter_mut()
                    .find(|s| s.name == subscription.name)
                {
                    sub.burned_training += subscription.balance as u64;
                } else {
                    month.subscriptions.push(SubscriptionStat {
                        name: subscription.name,
                        count: 0,
                        earned: 0,
                        burned_training: subscription.balance as u64,
                        discount: 0,
                    });
                }
            }
            Action::PayReward { .. }
            | Action::Payment { .. }
            | Action::PreSellSub { .. }
            | Action::FinalizedCanceledTraining { .. }
            | Action::FinalizedTraining { .. }
            | Action::Deposit { .. }
            | Action::CreateUser { .. }
            | Action::Freeze { .. }
            | Action::Unfreeze {}
            | Action::SignUp { .. }
            | Action::SignOut { .. }
            | Action::BlockUser { .. }
            | Action::ChangeBalance { .. }
            | Action::ChangeReservedBalance { .. }
            | Action::RemoveFamilyMember {}
            | Action::AddFamilyMember {} => {
                continue;
            }
            Action::ChangeSubscriptionDays { delta } => {
                continue;
            }
        }
    }

    Ok(())
}
