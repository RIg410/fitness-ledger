pub mod calendar;
pub mod history;
pub mod payment;
pub mod program;
pub mod requests;
pub mod rewards;
pub mod session;
pub mod subscription;
pub mod treasury;
pub mod user;
pub mod notification;

use bson::{doc, Bson};
use eyre::Result;
use futures_util::{StreamExt as _, TryStreamExt as _};
use history::HistoryStore;
use model::session::Session;
use mongodb::Collection;
use notification::NotificationStore;
use requests::RequestStore;
use rewards::RewardsStore;
use serde::{Deserialize, Serialize};
use session::Db;
use std::{collections::HashMap, sync::Arc};
use user::UserStore;

const DB_NAME: &str = "ledger_db";

#[derive(Clone)]
pub struct Storage {
    pub db: Arc<Db>,
    pub users: Arc<UserStore>,
    pub calendar: Arc<calendar::CalendarStore>,
    pub programs: Arc<program::ProgramStore>,
    pub treasury: Arc<treasury::TreasuryStore>,
    pub subscriptions: Arc<subscription::SubscriptionsStore>,
    pub history: Arc<HistoryStore>,
    pub rewards: Arc<RewardsStore>,
    pub requests: Arc<RequestStore>,
    pub notification: Arc<NotificationStore>,
}

impl Storage {
    pub async fn new(uri: &str) -> Result<Self> {
        let db = Db::new(uri, DB_NAME).await?;
        let users = UserStore::new(&db).await?;
        let calendar = calendar::CalendarStore::new(&db).await?;
        let programs = program::ProgramStore::new(&db);
        let treasury = treasury::TreasuryStore::new(&db).await?;
        let subscriptions = subscription::SubscriptionsStore::new(&db);
        let history = history::HistoryStore::new(&db).await?;
        let rewards = RewardsStore::new(&db).await?;
        let requests = RequestStore::new(&db).await?;
        let notification = NotificationStore::new(&db).await?;

        Ok(Storage {
            db: Arc::new(db),
            users: Arc::new(users),
            calendar: Arc::new(calendar),
            programs: Arc::new(programs),
            treasury: Arc::new(treasury),
            subscriptions: Arc::new(subscriptions),
            history: Arc::new(history),
            rewards: Arc::new(rewards),
            requests: Arc::new(requests),
            notification: Arc::new(notification),
        })
    }

    pub async fn backup(&self, session: &mut Session) -> Result<HashMap<String, CollectionBackup>> {
        let mut collections = self.db.list_collections().await?;

        let mut backup = HashMap::new();
        while let Some(collection) = collections.next().await {
            let collection = collection?;
            let name = collection.name.clone();
            let collections: Collection<Bson> = self.db.collection(&name);
            let mut items = collections.find(doc! {}).session(&mut *session).await?;
            let items: Vec<Bson> = items.stream(&mut *session).try_collect().await?;
            backup.insert(name, CollectionBackup { data: items });
        }
        Ok(backup)
    }

    pub async fn restore(
        &self,
        backup: HashMap<String, CollectionBackup>,
        session: &mut Session,
    ) -> Result<()> {
        for (name, items) in backup {
            let collections: Collection<Bson> = self.db.collection(&name);
            collections
                .delete_many(doc! {})
                .session(&mut *session)
                .await?;
            let items = items.data;
            for item in items {
                collections.insert_one(item).session(&mut *session).await?;
            }
        }
        Ok(())
    }
}

#[derive(Serialize, Deserialize)]
pub struct CollectionBackup {
    data: Vec<Bson>,
}
