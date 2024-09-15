use std::sync::Arc;

use chrono::{DateTime, Local, Utc};
use eyre::Result;
use model::{
    decimal::Decimal,
    log::{Action, LogEntry},
    program::Program,
    rights::Rule,
    session::Session,
    subscription::Subscription,
    training::Training,
    treasury::Sell,
    user::UserName,
};
use mongodb::bson::oid::ObjectId;
use storage::logs::LogStore;

#[derive(Clone)]
pub struct Logs {
    store: Arc<LogStore>,
}

impl Logs {
    pub fn new(store: LogStore) -> Self {
        Logs {
            store: Arc::new(store),
        }
    }

    pub async fn logs(
        &self,
        session: &mut Session,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<LogEntry>> {
        self.store.get_logs(session, limit, offset).await
    }

    pub async fn create_user(
        &self,
        session: &mut Session,
        tg_id: i64,
        name: UserName,
        phone: String,
    ) {
        let entry = model::log::LogEntry {
            actor: session.actor(),
            date_time: chrono::Local::now().with_timezone(&Utc),
            action: Action::CreateUser {
                tg_id,
                name: name,
                phone: phone,
            },
        };
        if let Err(err) = self.store.store(session, entry).await {
            log::error!("Failed to store log entry: {}", err);
        }
    }

    pub async fn set_user_birthday(
        &self,
        session: &mut Session,
        tg_id: i64,
        birthday: DateTime<chrono::Local>,
    ) {
        let entry = model::log::LogEntry {
            actor: session.actor(),
            date_time: chrono::Local::now().with_timezone(&Utc),
            action: Action::SetUserBirthday {
                tg_id,
                birthday: birthday.with_timezone(&Utc),
            },
        };
        if let Err(err) = self.store.store(session, entry).await {
            log::error!("Failed to store log entry: {}", err);
        }
    }

    pub async fn gc(
        &self,
        session: &mut Session,
        date_time: DateTime<chrono::Local>,
    ) -> Result<u64> {
        self.store.gc(session, date_time).await
    }

    pub async fn edit_user_rule(
        &self,
        session: &mut Session,
        tg_id: i64,
        rule: Rule,
        is_active: bool,
    ) {
        let entry = model::log::LogEntry {
            actor: session.actor(),
            date_time: chrono::Local::now().with_timezone(&Utc),
            action: Action::EditUserRule {
                tg_id,
                rule,
                is_active,
            },
        };
        if let Err(err) = self.store.store(session, entry).await {
            log::error!("Failed to store log entry: {}", err);
        }
    }

    pub async fn freeze(&self, session: &mut Session, tg_id: i64, days: u32) {
        let entry = model::log::LogEntry {
            actor: session.actor(),
            date_time: chrono::Local::now().with_timezone(&Utc),
            action: Action::Freeze { tg_id, days },
        };
        if let Err(err) = self.store.store(session, entry).await {
            log::error!("Failed to store log entry: {}", err);
        }
    }

    pub(crate) async fn unfreeze(&self, session: &mut Session, tg_id: i64) {
        let entry = model::log::LogEntry {
            actor: session.actor(),
            date_time: chrono::Local::now().with_timezone(&Utc),
            action: Action::Unfreeze { tg_id },
        };
        if let Err(err) = self.store.store(session, entry).await {
            log::error!("Failed to store log entry: {}", err);
        }
    }

    pub async fn change_balance(&self, session: &mut Session, tg_id: i64, amount: i32) {
        let entry = model::log::LogEntry {
            actor: session.actor(),
            date_time: chrono::Local::now().with_timezone(&Utc),
            action: Action::ChangeBalance { tg_id, amount },
        };
        if let Err(err) = self.store.store(session, entry).await {
            log::error!("Failed to store log entry: {}", err);
        }
    }

    pub async fn set_user_name(
        &self,
        session: &mut Session,
        tg_id: i64,
        first_name: &str,
        last_name: &str,
    ) {
        let entry = model::log::LogEntry {
            actor: session.actor(),
            date_time: chrono::Local::now().with_timezone(&Utc),
            action: Action::SetUserName {
                tg_id,
                first_name: first_name.to_string(),
                last_name: last_name.to_string(),
            },
        };
        if let Err(err) = self.store.store(session, entry).await {
            log::error!("Failed to store log entry: {}", err);
        }
    }
    pub async fn sell(&self, session: &mut Session, seller: ObjectId, buyer: ObjectId, sell: Sell) {
        let entry = model::log::LogEntry {
            actor: session.actor(),
            date_time: chrono::Local::now().with_timezone(&Utc),
            action: Action::Sell {
                seller,
                buyer,
                sell,
            },
        };
        if let Err(err) = self.store.store(session, entry).await {
            log::error!("Failed to store log entry: {}", err);
        }
    }

    pub async fn payment(
        &self,
        session: &mut Session,
        user: ObjectId,
        amount: Decimal,
        description: String,
        date_time: &chrono::DateTime<Local>,
    ) {
        let entry = model::log::LogEntry {
            actor: session.actor(),
            date_time: chrono::Local::now().with_timezone(&Utc),
            action: Action::Payment {
                user,
                amount,
                description,
                date_time: date_time.with_timezone(&Utc),
            },
        };
        if let Err(err) = self.store.store(session, entry).await {
            log::error!("Failed to store log entry: {}", err);
        }
    }

    pub async fn deposit(
        &self,
        session: &mut Session,
        user: ObjectId,
        amount: Decimal,
        description: String,
        date_time: &chrono::DateTime<Local>,
    ) {
        let entry = model::log::LogEntry {
            actor: session.actor(),
            date_time: chrono::Local::now().with_timezone(&Utc),
            action: Action::Deposit {
                user,
                amount,
                description,
                date_time: date_time.with_timezone(&Utc),
            },
        };
        if let Err(err) = self.store.store(session, entry).await {
            log::error!("Failed to store log entry: {}", err);
        }
    }

    pub async fn delete_sub(&self, session: &mut Session, sub: Subscription) {
        let entry = model::log::LogEntry {
            actor: session.actor(),
            date_time: chrono::Local::now().with_timezone(&Utc),
            action: Action::DeleteSub { sub },
        };
        if let Err(err) = self.store.store(session, entry).await {
            log::error!("Failed to store log entry: {}", err);
        }
    }

    pub async fn create_sub(&self, session: &mut Session, sub: Subscription) {
        let entry = model::log::LogEntry {
            actor: session.actor(),
            date_time: chrono::Local::now().with_timezone(&Utc),
            action: Action::CreateSub { sub },
        };
        if let Err(err) = self.store.store(session, entry).await {
            log::error!("Failed to store log entry: {}", err);
        }
    }

    pub async fn create_program(&self, session: &mut Session, program: Program) {
        let entry = model::log::LogEntry {
            actor: session.actor(),
            date_time: chrono::Local::now().with_timezone(&Utc),
            action: Action::CreateProgram { program },
        };
        if let Err(err) = self.store.store(session, entry).await {
            log::error!("Failed to store log entry: {}", err);
        }
    }

    pub async fn sell_free_subscription(
        &self,
        session: &mut Session,
        price: Decimal,
        item: u32,
        buyer: i64,
        seller: i64,
    ) {
        let entry = model::log::LogEntry {
            actor: session.actor(),
            date_time: chrono::Local::now().with_timezone(&Utc),
            action: Action::FreeSellSub {
                seller,
                buyer,
                price,
                item,
            },
        };
        if let Err(err) = self.store.store(session, entry).await {
            log::error!("Failed to store log entry: {}", err);
        }
    }

    pub async fn sell_subscription(
        &self,
        session: &mut Session,
        subscription: Subscription,
        buyer: i64,
        seller: i64,
    ) {
        let entry = model::log::LogEntry {
            actor: session.actor(),
            date_time: chrono::Local::now().with_timezone(&Utc),
            action: Action::SellSub {
                seller,
                buyer,
                subscription,
            },
        };
        if let Err(err) = self.store.store(session, entry).await {
            log::error!("Failed to store log entry: {}", err);
        }
    }

    pub async fn sign_out(&self, session: &mut Session, training: Training, tg_id: i64) {
        let entry = model::log::LogEntry {
            actor: session.actor(),
            date_time: chrono::Local::now().with_timezone(&Utc),
            action: Action::SignOut {
                name: training.name,
                id: training.id,
                proto_id: training.proto_id,
                start_at: training.start_at,
                user_id: tg_id,
            },
        };

        if let Err(err) = self.store.store(session, entry).await {
            log::error!("Failed to store log entry: {}", err);
        }
    }

    pub async fn sign_up(&self, session: &mut Session, training: Training, tg_id: i64) {
        let entry = model::log::LogEntry {
            actor: session.actor(),
            date_time: chrono::Local::now().with_timezone(&Utc),
            action: Action::SignUp {
                name: training.name,
                id: training.id,
                proto_id: training.proto_id,
                start_at: training.start_at,
                user_id: tg_id,
            },
        };

        if let Err(err) = self.store.store(session, entry).await {
            log::error!("Failed to store log entry: {}", err);
        }
    }

    pub async fn block_user(&self, session: &mut Session, tg_id: i64, is_active: bool) {
        let entry = model::log::LogEntry {
            actor: session.actor(),
            date_time: chrono::Local::now().with_timezone(&Utc),
            action: Action::BlockUser { tg_id, is_active },
        };

        if let Err(err) = self.store.store(session, entry).await {
            log::error!("Failed to store log entry: {}", err);
        }
    }

    pub async fn cancel_training(&self, session: &mut Session, training: &Training) {
        let entry = model::log::LogEntry {
            actor: session.actor(),
            date_time: chrono::Local::now().with_timezone(&Utc),
            action: Action::CancelTraining {
                name: training.name.clone(),
                id: training.id,
                proto_id: training.proto_id,
                start_at: training.start_at,
            },
        };

        if let Err(err) = self.store.store(session, entry).await {
            log::error!("Failed to store log entry: {}", err);
        }
    }

    pub async fn restore_training(&self, session: &mut Session, training: &Training) {
        let entry = model::log::LogEntry {
            actor: session.actor(),
            date_time: chrono::Local::now().with_timezone(&Utc),
            action: Action::RestoreTraining {
                name: training.name.clone(),
                id: training.id,
                proto_id: training.proto_id,
                start_at: training.start_at,
            },
        };

        if let Err(err) = self.store.store(session, entry).await {
            log::error!("Failed to store log entry: {}", err);
        }
    }

    pub async fn delete_training(&self, session: &mut Session, training: &Training, all: bool) {
        let entry = model::log::LogEntry {
            actor: session.actor(),
            date_time: chrono::Local::now().with_timezone(&Utc),
            action: Action::DeleteTraining {
                name: training.name.clone(),
                id: training.id,
                proto_id: training.proto_id,
                start_at: training.start_at,
                all,
            },
        };

        if let Err(err) = self.store.store(session, entry).await {
            log::error!("Failed to store log entry: {}", err);
        }
    }

    pub async fn schedule(&self, session: &mut Session, training: &Training) {
        let entry = model::log::LogEntry {
            actor: session.actor(),
            date_time: chrono::Local::now().with_timezone(&Utc),
            action: Action::Schedule {
                name: training.name.clone(),
                id: training.id,
                proto_id: training.proto_id,
                start_at: training.start_at,
                instructor: training.instructor,
            },
        };

        if let Err(err) = self.store.store(session, entry).await {
            log::error!("Failed to store log entry: {}", err);
        }
    }

    pub async fn process_finished(&self, session: &mut Session, training: &Training) {
        let entry = model::log::LogEntry {
            actor: session.actor(),
            date_time: chrono::Local::now().with_timezone(&Utc),
            action: Action::FinalizedTraining {
                name: training.name.clone(),
                id: training.id,
                proto_id: training.proto_id,
                start_at: training.start_at,
                clients: training.clients.clone(),
                instructor: training.instructor,
            },
        };

        if let Err(err) = self.store.store(session, entry).await {
            log::error!("Failed to store log entry: {}", err);
        }
    }

     pub async fn process_canceled(&self, session: &mut Session, training: &Training) {
        let entry = model::log::LogEntry {
            actor: session.actor(),
            date_time: chrono::Local::now().with_timezone(&Utc),
            action: Action::FinalizedCanceledTraining {
                name: training.name.clone(),
                id: training.id,
                proto_id: training.proto_id,
                start_at: training.start_at,
                clients: training.clients.clone(),
                instructor: training.instructor,
            },
        };

        if let Err(err) = self.store.store(session, entry).await {
            log::error!("Failed to store log entry: {}", err);
        }
    }
}
