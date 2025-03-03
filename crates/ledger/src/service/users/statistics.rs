use std::time::Duration;

use bson::oid::ObjectId;
use chrono::{DateTime, Local, Utc};
use model::{
    session::Session,
    statistics::{month::SubscriptionStat, user::Statistics},
};

use super::Users;

impl Users {
    pub async fn collect_statistics(
        &self,
        session: &mut Session,
        user: &ObjectId,
    ) -> Result<Statistics, eyre::Error> {
        let statistics = Statistics::default();
        let history = self.logs.get_actor_logs(session, *user, None, 0).await?;

        for row in history {
            match row.action {
                model::history::Action::SignUp { .. }
                | model::history::Action::BlockUser { .. } => {
                    //no-op
                }
                model::history::Action::SignOut { .. } => {
                    // if row.date_time.with_timezone(&Local) + Duration::from_mins(180) > start_at {
                    //     statistics
                    //         .training
                    //         .entry(name)
                    //         .or_default()
                    //         .cancellations_count += 1;
                    // }
                }
                model::history::Action::SellSub { .. } => {
                    // let mut stat = statistics
                    //     .subscriptions
                    //     .entry(subscription.id)
                    //     .or_insert_with(|| SubscriptionStat::new(subscription.name.clone()));
                    // stat.soult_count += 1;

                    // let price = subscription.pric
                }
                model::history::Action::PreSellSub { .. } => todo!(),
                model::history::Action::FinalizedCanceledTraining { .. } => todo!(),
                model::history::Action::FinalizedTraining { .. } => todo!(),
                model::history::Action::Payment { .. } => todo!(),
                model::history::Action::Deposit { .. } => todo!(),
                model::history::Action::CreateUser { .. } => todo!(),
                model::history::Action::Freeze { .. } => todo!(),
                model::history::Action::Unfreeze {} => todo!(),
                model::history::Action::ChangeBalance { .. } => todo!(),
                model::history::Action::ChangeReservedBalance { .. } => todo!(),
                model::history::Action::PayReward { .. } => todo!(),
                model::history::Action::ExpireSubscription { .. } => todo!(),
                model::history::Action::BuySub { .. } => todo!(),
                model::history::Action::RemoveFamilyMember {} => todo!(),
                model::history::Action::AddFamilyMember {} => todo!(),
                model::history::Action::ChangeSubscriptionDays { .. } => todo!(),
            }
        }

        Ok(statistics)
    }
}
