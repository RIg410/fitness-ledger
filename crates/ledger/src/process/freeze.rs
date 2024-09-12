use crate::Ledger;
use chrono::Local;
use eyre::Result;
use log::{info, warn};
use mongodb::ClientSession;

pub struct FreezeBg {
    ledger: Ledger,
}

impl FreezeBg {
    pub fn new(ledger: Ledger) -> FreezeBg {
        FreezeBg { ledger }
    }

    pub async fn process(&self, session: &mut ClientSession) -> Result<()> {
        let users = self.ledger.users.find_users_to_unfreeze(session).await?;
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
            self.ledger.users.unfreeze(session, user.tg_id).await?;
        }
        Ok(())
    }
}
