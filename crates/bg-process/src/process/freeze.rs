use std::sync::Arc;

use crate::{Ledger, Task};
use async_trait::async_trait;
use chrono::Local;
use eyre::{Error, Result};
use log::{info, warn};

#[derive(Clone)]
pub struct FreezeBg {
    ledger: Arc<Ledger>,
}

#[async_trait]
impl Task for FreezeBg {
    const NAME: &'static str = "freeze";
    const CRON: &'static str = "every 1 hour";

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
            self.ledger.users.unfreeze(&mut session, user.id).await?;
        }
        Ok(())
    }
}

impl FreezeBg {
    pub fn new(ledger: Arc<Ledger>) -> FreezeBg {
        FreezeBg { ledger }
    }
}
