use crate::{Ledger, Task};
use async_trait::async_trait;
use bot_core::bot::TgBot;
use eyre::Error;
use log::info;
use std::sync::Arc;
use teloxide::{prelude::Requester as _, types::ChatId};

#[derive(Clone)]
pub struct UserNameSync {
    ledger: Arc<Ledger>,
    bot: Arc<TgBot>,
}

impl UserNameSync {
    pub fn new(ledger: Arc<Ledger>, bot: Arc<TgBot>) -> UserNameSync {
        UserNameSync { ledger, bot }
    }
}

#[async_trait]
impl Task for UserNameSync {
    const NAME: &'static str = "user name sync";
    const CRON: &'static str = "every day at 03:00";

    async fn process(&mut self) -> Result<(), Error> {
        let mut session = self.ledger.db.start_session().await?;
        info!("start user name sync");
        let mut cursor = self.ledger.users.find_all(&mut session, None, None).await?;
        while let Some(user) = cursor.next(&mut session).await {
            let user = user?;
            if user.tg_id > 0 {
                if let Some(username) = self.bot.get_chat(ChatId(user.tg_id)).await?.username() {
                    match user.name.tg_user_name {
                        Some(current_name) => {
                            if current_name != username {
                                info!("update tg user_name:{} {}", user.id, username);
                                self.ledger
                                    .users
                                    .set_tg_user_name(&mut session, user.id, username)
                                    .await?;
                            }
                        }
                        None => {
                            info!("update tg user_name:{} {}", user.id, username);
                            self.ledger
                                .users
                                .set_tg_user_name(&mut session, user.id, username)
                                .await?;
                        }
                    }
                }
            }
        }

        Ok(())
    }
}
