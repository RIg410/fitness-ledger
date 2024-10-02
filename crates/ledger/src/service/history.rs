use std::{ops::Deref, sync::Arc};

use chrono::{DateTime, Local, Utc};
use eyre::Result;
use model::{
    decimal::Decimal,
    history::{Action, HistoryRow},
    session::Session,
    subscription::{Subscription, UserSubscription},
    training::Training,
    user::UserName,
};
use mongodb::bson::oid::ObjectId;
use storage::history::HistoryStore;

#[derive(Clone)]
pub struct History {
    store: Arc<HistoryStore>,
}

impl History {
    pub fn new(store: HistoryStore) -> Self {
        History {
            store: Arc::new(store),
        }
    }

    pub async fn expire_subscription(
        &self,
        session: &mut Session,
        id: ObjectId,
        subscription: UserSubscription,
    ) -> Result<()> {
        let entry = HistoryRow::new(id, Action::ExpireSubscription { subscription });
        self.store.store(session, entry).await
    }

    pub async fn pay_reward(
        &self,
        session: &mut Session,
        user: ObjectId,
        amount: Decimal,
    ) -> Result<()> {
        let entry =
            HistoryRow::with_sub_actors(session.actor(), vec![user], Action::PayReward { amount });
        self.store.store(session, entry).await
    }

    pub async fn logs(
        &self,
        session: &mut Session,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<HistoryRow>> {
        self.store.get_logs(session, limit, offset).await
    }

    pub async fn actor_logs(
        &self,
        session: &mut Session,
        actor: ObjectId,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<HistoryRow>> {
        self.store
            .get_actor_logs(session, actor, limit, offset)
            .await
    }

    pub async fn create_user(
        &self,
        session: &mut Session,
        name: UserName,
        phone: String,
    ) -> Result<()> {
        let entry = HistoryRow::new(session.actor(), Action::CreateUser { name, phone });
        self.store.store(session, entry).await
    }

    pub async fn freeze(&self, session: &mut Session, user: ObjectId, days: u32) -> Result<()> {
        let entry =
            HistoryRow::with_sub_actors(session.actor(), vec![user], Action::Freeze { days });
        self.store.store(session, entry).await
    }

    pub async fn unfreeze(&self, session: &mut Session, user: ObjectId) -> Result<()> {
        let entry = HistoryRow::with_sub_actors(session.actor(), vec![user], Action::Unfreeze {});
        self.store.store(session, entry).await
    }

    pub async fn change_balance(
        &self,
        session: &mut Session,
        user: ObjectId,
        amount: i32,
    ) -> Result<()> {
        let entry = HistoryRow::with_sub_actors(
            session.actor(),
            vec![user],
            Action::ChangeBalance { amount },
        );
        self.store.store(session, entry).await
    }

    pub async fn change_reserved_balance(
        &self,
        session: &mut Session,
        user: ObjectId,
        amount: i32,
    ) -> Result<()> {
        let entry = HistoryRow::with_sub_actors(
            session.actor(),
            vec![user],
            Action::ChangeReservedBalance { amount },
        );
        self.store.store(session, entry).await
    }

    pub async fn sell_subscription(
        &self,
        session: &mut Session,
        subscription: Subscription,
        buyer: ObjectId,
    ) -> Result<()> {
        let entry = HistoryRow::with_sub_actors(
            session.actor(),
            vec![buyer],
            Action::SellSub { subscription },
        );
        self.store.store(session, entry).await
    }

    pub async fn presell_subscription(
        &self,
        session: &mut Session,
        subscription: Subscription,
        buyer: String,
    ) -> Result<()> {
        let entry = HistoryRow::new(
            session.actor(),
            Action::PreSellSub {
                subscription,
                phone: buyer,
            },
        );
        self.store.store(session, entry).await
    }

    pub async fn sign_up(
        &self,
        session: &mut Session,
        user_id: ObjectId,
        start_at: DateTime<Local>,
        name: String,
    ) -> Result<()> {
        self.store
            .store(
                session,
                HistoryRow::with_sub_actors(
                    session.actor(),
                    vec![user_id],
                    Action::SignUp { start_at, name },
                ),
            )
            .await
    }

    pub async fn sign_out(
        &self,
        session: &mut Session,
        user_id: ObjectId,
        start_at: DateTime<Local>,
        name: String,
    ) -> Result<()> {
        self.store
            .store(
                session,
                HistoryRow::with_sub_actors(
                    session.actor(),
                    vec![user_id],
                    Action::SignOut { start_at, name },
                ),
            )
            .await
    }

    pub async fn block_user(
        &self,
        session: &mut Session,
        user: ObjectId,
        is_active: bool,
    ) -> Result<()> {
        self.store
            .store(
                session,
                HistoryRow::with_sub_actors(
                    session.actor(),
                    vec![user],
                    Action::BlockUser { is_active },
                ),
            )
            .await
    }

    pub async fn process_finished(&self, session: &mut Session, training: &Training) -> Result<()> {
        let sub_actors = training.clients.to_vec();

        let entry = HistoryRow::with_sub_actors(
            training.instructor,
            sub_actors,
            Action::FinalizedTraining {
                name: training.name.clone(),
                start_at: training.start_at,
            },
        );
        self.store.store(session, entry).await
    }

    pub async fn process_canceled(&self, session: &mut Session, training: &Training) -> Result<()> {
        let sub_actors = training.clients.to_vec();

        let entry = HistoryRow::with_sub_actors(
            training.instructor,
            sub_actors,
            Action::FinalizedCanceledTraining {
                name: training.name.clone(),
                start_at: training.start_at,
            },
        );
        self.store.store(session, entry).await
    }

    pub async fn payment(
        &self,
        session: &mut Session,
        amount: Decimal,
        description: String,
        date_time: &DateTime<Local>,
    ) -> Result<()> {
        let entry = HistoryRow::new(
            session.actor(),
            Action::Payment {
                amount,
                description,
                date_time: date_time.with_timezone(&Utc),
            },
        );
        self.store.store(session, entry).await
    }

    pub async fn deposit(
        &self,
        session: &mut Session,
        amount: Decimal,
        description: String,
        date_time: &DateTime<Local>,
    ) -> Result<()> {
        let entry = HistoryRow::new(
            session.actor(),
            Action::Deposit {
                amount,
                description,
                date_time: date_time.with_timezone(&Utc),
            },
        );
        self.store.store(session, entry).await
    }
}

impl Deref for History {
    type Target = HistoryStore;

    fn deref(&self) -> &Self::Target {
        &self.store
    }
}