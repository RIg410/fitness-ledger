use crate::{Ledger, Task};
use async_trait::async_trait;
use chrono::Local;
use eyre::{Error, Result};
use log::{info, warn};

#[derive(Clone)]
pub struct FreezeBg {
    ledger: Ledger,
}

#[async_trait]
impl Task for FreezeBg {
    const NAME: &'static str = "freeze";
    const CRON: &'static str = "every day at 00:00";

    async fn process(&mut self) -> Result<(), Error> {
        let mut session = self.ledger.db.start_session().await?;

        let users = self
            .ledger
            .users
            .find_users_to_unfreeze(&mut session)
            .await?;
        let now = Local::now();
        for user in users {
            let freeze = if let Some(freeze) = user.freeze.as_ref() {
                freeze
            } else {
                warn!("User {} has no freeze", user.tg_id);
                continue;
            };
            if freeze.freeze_end > now {
                warn!("User {} has not expired freeze", user.tg_id);
                continue;
            }
            info!("Unfreezing user {}", user.tg_id);
            self.ledger.users.unfreeze(&mut session, user.tg_id).await?;
        }
        Ok(())
    }
}

impl FreezeBg {
    pub fn new(ledger: Ledger) -> FreezeBg {
        FreezeBg { ledger }
    }
}
