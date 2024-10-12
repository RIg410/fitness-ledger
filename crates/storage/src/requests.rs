use std::sync::Arc;

use bson::doc;
use chrono::{DateTime, Local};
use eyre::Error;
use model::{request::Request, session::Session};
use mongodb::{Collection, IndexModel, SessionCursor};

const COLLECTION: &str = "requests";

#[derive(Clone)]
pub struct RequestStore {
    requests: Arc<Collection<Request>>,
}

impl RequestStore {
    pub async fn new(db: &mongodb::Database) -> Result<Self, Error> {
        let reward = db.collection(COLLECTION);
        reward
            .create_index(IndexModel::builder().keys(doc! { "phone": 1 }).build())
            .await?;
        reward
            .create_index(
                IndexModel::builder()
                    .keys(doc! { "created_at": -1 })
                    .build(),
            )
            .await?;
        Ok(RequestStore {
            requests: Arc::new(reward),
        })
    }

    pub async fn update(&self, session: &mut Session, request: Request) -> Result<(), Error> {
        self.requests
            .replace_one(doc! { "_id": request.id }, request)
            .session(&mut *session)
            .await?;
        Ok(())
    }

    pub async fn add(&self, session: &mut Session, request: Request) -> Result<(), Error> {
        self.requests
            .insert_one(request)
            .session(&mut *session)
            .await?;
        Ok(())
    }

    pub async fn get_by_phone(
        &self,
        session: &mut Session,
        phone: &str,
    ) -> Result<Option<Request>, Error> {
        let request = self
            .requests
            .find_one(doc! { "phone": phone })
            .session(&mut *session)
            .await?;
        Ok(request)
    }

    pub async fn cursor(
        &self,
        session: &mut Session,
        from: Option<DateTime<Local>>,
        to: Option<DateTime<Local>>,
    ) -> Result<SessionCursor<Request>, Error> {
        let mut query = doc! {};
        if let Some(from) = from {
            query.insert("created_at", doc! { "$gte": from });
        }
        if let Some(to) = to {
            query.insert("created_at", doc! { "$lt": to });
        }
        let cursor = self.requests.find(query).session(&mut *session).await?;
        Ok(cursor)
    }

    pub async fn get_all_page(
        &self,
        session: &mut Session,
        limit: i64,
        offset: u64,
    ) -> Result<Vec<Request>, Error> {
        let mut cursor = self
            .requests
            .find(doc! {})
            .session(&mut *session)
            .skip(offset)
            .limit(limit)
            .await?;

        let mut requests = Vec::new();
        while let Some(request) = cursor.next(&mut *session).await {
            requests.push(request?);
        }
        Ok(requests)
    }

    pub async fn dump(&self, session: &mut Session) -> Result<Vec<Request>, Error> {
        let mut cursor = self.requests.find(doc! {}).session(&mut *session).await?;
        let mut rewards = Vec::new();
        while let Some(reward) = cursor.next(&mut *session).await {
            rewards.push(reward?);
        }
        Ok(rewards)
    }
}
