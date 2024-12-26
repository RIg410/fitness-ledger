use async_trait::async_trait;
use bson::doc;
use eyre::Error;
use futures_util::TryStreamExt;
use model::session::Session;
use serde::{de::DeserializeOwned, Serialize};

#[async_trait]
pub trait Backup<Item> {
    async fn dump(&self, session: &mut Session) -> Result<Vec<Item>, Error>;
    async fn restore(&self, item: Vec<Item>, session: &mut Session) -> Result<(), Error>;
}

pub trait Collection<Item>
where
    Item: Serialize + DeserializeOwned + Send + Sync + 'static,
{
    fn collection(&self) -> &mongodb::Collection<Item>;
}

#[async_trait]
impl<Item, C> Backup<Item> for C
where
    Item: Serialize + DeserializeOwned + Send + Sync + 'static,
    C: Collection<Item> + Send + Sync,
{
    async fn dump(&self, session: &mut Session) -> Result<Vec<Item>, Error> {
        let mut cursor = self
            .collection()
            .find(doc! {})
            .session(&mut *session)
            .await?;
        let items = cursor.stream(&mut *session).try_collect().await?;
        Ok(items)
    }

    async fn restore(&self, items: Vec<Item>, session: &mut Session) -> Result<(), Error> {
        let collection = self.collection();
        collection
            .delete_many(doc! {})
            .session(&mut *session)
            .await?;

        for item in items {
            collection.insert_one(item).session(&mut *session).await?;
        }
        Ok(())
    }
}
