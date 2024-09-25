use bson::doc;
use eyre::Result;
use futures_util::TryStreamExt as _;
use model::{session::Session, user::UserPreSell};
use mongodb::{Collection, Database};
use std::sync::Arc;

const PRESELL_COLLECTION: &str = "users_presell";

#[derive(Clone)]
pub struct PreSellStore {
    pub(crate) pre_cell: Arc<Collection<UserPreSell>>,
}

impl PreSellStore {
    pub(crate) async fn new(db: &Database) -> Result<Self> {
        Ok(PreSellStore {
            pre_cell: Arc::new(db.collection(PRESELL_COLLECTION)),
        })
    }

    pub async fn add(&self, session: &mut Session, user: UserPreSell) -> Result<()> {
        self.pre_cell.insert_one(user).session(session).await?;
        Ok(())
    }

    pub async fn get(&self, session: &mut Session, phone: &String) -> Result<Option<UserPreSell>> {
        let user = self
            .pre_cell
            .find_one(doc! {"phone": phone})
            .session(session)
            .await?;
        Ok(user)
    }

    pub async fn delete(&self, session: &mut Session, phone: &String) -> Result<()> {
        self.pre_cell
            .delete_one(doc! {"phone": phone})
            .session(session)
            .await?;
        Ok(())
    }

    pub async fn dump(&self, session: &mut Session) -> Result<Vec<UserPreSell>> {
        let mut cursor = self.pre_cell.find(doc! {}).session(&mut *session).await?;
        Ok(cursor.stream(&mut *session).try_collect().await?)
    }
}
