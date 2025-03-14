use bson::oid::ObjectId;
use chrono::{Duration, Local};
use model::{
    decimal::Decimal,
    session::Session,
    statistics::user::{Statistics, SubscriptionStat},
    training::CLOSE_SING_UP,
};

use super::Users;

impl Users {
    pub async fn collect_statistics(
        &self,
        session: &mut Session,
        user: &ObjectId,
    ) -> Result<Statistics, eyre::Error> {
        let mut statistics = Statistics::default();
        let history = self.logs.get_actor_logs(session, *user, None, 0).await?;

        for row in history {
            match row.action {
                model::history::Action::RemoveFamilyMember {}
                | model::history::Action::AddFamilyMember {}
                | model::history::Action::PayReward { .. }
                | model::history::Action::Unfreeze {}
                | model::history::Action::Deposit { .. }
                | model::history::Action::CreateUser { .. }
                | model::history::Action::Payment { .. }
                | model::history::Action::FinalizedCanceledTraining { .. }
                | model::history::Action::PreSellSub { .. }
                | model::history::Action::SignUp { .. }
                | model::history::Action::BlockUser { .. } => {
                    //no-op
                }
                model::history::Action::SignOut { start_at, name, .. } => {
                    if row.date_time.with_timezone(&Local) + Duration::minutes(CLOSE_SING_UP as i64)
                        > start_at
                    {
                        statistics
                            .training
                            .entry(name)
                            .or_default()
                            .cancellations_count += 1;
                    }
                }
                model::history::Action::SellSub {
                    subscription,
                    discount,
                } => {
                    let stat = statistics
                        .subscriptions
                        .entry(subscription.id)
                        .or_insert_with(|| SubscriptionStat::new(subscription.name.clone()));
                    stat.soult_count += 1;

                    if let Some(discount) = discount {
                        let discount_sum = subscription.price * discount;
                        stat.discount += discount_sum;
                        stat.spent += subscription.price - discount_sum;
                    } else {
                        stat.spent += subscription.price;
                    }
                }
                model::history::Action::FinalizedTraining { name, .. } => {
                    let training = statistics.training.entry(name).or_default();
                    training.count += 1;
                }
                model::history::Action::Freeze { days } => {
                    statistics.total_freeze += days;
                }
                model::history::Action::ChangeBalance { amount } => {
                    statistics.changed_subscription_balance += amount as i64;
                }
                model::history::Action::ChangeReservedBalance { amount } => {
                    statistics.changed_subscription_balance += amount as i64;
                }
                model::history::Action::ChangeSubscriptionDays { delta } => {
                    statistics.changed_subscription_days += delta as i64;
                }
                model::history::Action::ExpireSubscription { subscription } => {
                    let stat = statistics
                        .subscriptions
                        .entry(subscription.subscription_id)
                        .or_insert_with(|| SubscriptionStat::new(subscription.name.clone()));

                    stat.expired_sum +=
                        subscription.item_price() * Decimal::int(subscription.balance as i64);
                    stat.expired_trainings += subscription.balance as u64;
                }
            }
        }

        Ok(statistics)
    }
}
