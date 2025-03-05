use eyre::Error;
use model::{history::Action, session::Session, user::User};

use super::Statistics;

impl Statistics {
    pub async fn find_clients_without_subscription(
        &self,
        session: &mut Session,
    ) -> Result<Vec<User>, Error> {
        let mut users = self.users.users_without_subscription(session).await?;

        let mut result = vec![];
        while let Some(user) = users.next(session).await {
            let user = user?;

            if user.has_subscriptions()
                || user.phone.is_none()
                || !user.is_active
                || user.employee.is_some()
            {
                continue;
            }

            let logs = self
                .history
                .get_actor_logs(session, user.id, None, 0)
                .await?;
            let has_actions = logs.iter().any(|log| match &log.action {
                Action::SellSub {
                    subscription,
                    discount: _,
                }
                | Action::BuySub {
                    subscription,
                    discount: _,
                } => subscription.items > 1 && subscription.subscription_type.is_group(),
                _ => false,
            });
            if has_actions {
                result.push(user);
            }
        }
        Ok(result)
    }
}
