use crate::session::Db;
use bson::doc;
use eyre::Error;
use model::{auth::AuthKey, session::Session};
use mongodb::{Collection, IndexModel};

const COLLECTION: &str = "auth_keys";

pub struct AuthKeys {
    pub(crate) days: Collection<AuthKey>,
}

impl AuthKeys {
    pub async fn new(db: &Db) -> Result<Self, Error> {
        let store = db.collection(COLLECTION);
        store
            .create_index(IndexModel::builder().keys(doc! { "key": -1 }).build())
            .await?;
        Ok(AuthKeys { days: store })
    }

    pub async fn insert(&self, session: &mut Session, key: &AuthKey) -> Result<(), Error> {
        self.days.insert_one(key).session(session).await?;
        Ok(())
    }

    pub async fn get_by_key(
        &self,
        session: &mut Session,
        key: &str,
    ) -> Result<Option<AuthKey>, Error> {
        Ok(self
            .days
            .find_one(doc! { "key": key })
            .session(session)
            .await?)
    }

    pub async fn get(
        &self,
        session: &mut Session,
        id: bson::oid::ObjectId,
    ) -> Result<Option<AuthKey>, Error> {
        Ok(self
            .days
            .find_one(doc! { "_id": id })
            .session(session)
            .await?)
    }
}
