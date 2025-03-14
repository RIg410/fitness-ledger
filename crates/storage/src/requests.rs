use bson::{doc, oid::ObjectId, Regex};
use chrono::{DateTime, Local, Utc};
use eyre::Error;
use model::{request::Request, session::Session};
use mongodb::{Collection, IndexModel, SessionCursor};

const COLLECTION: &str = "requests";

pub struct RequestStore {
    pub(crate) store: Collection<Request>,
}

impl RequestStore {
    pub async fn new(db: &mongodb::Database) -> Result<Self, Error> {
        let reward = db.collection(COLLECTION);
        reward
            .create_index(IndexModel::builder().keys(doc! { "phone": 1 }).build())
            .await?;
        reward
            .create_index(IndexModel::builder().keys(doc! { "created": -1 }).build())
            .await?;
        Ok(RequestStore { store: reward })
    }

    pub async fn find_by_words(
        &self,
        session: &mut Session,
        words: Vec<&str>,
    ) -> Result<Vec<Request>, Error> {
        let pattern = format!("({})", words.join("|"));

        let query = doc! {
            "$or": [
                { "comment": {
                    "$regex": Regex {
                        pattern: pattern.clone(),
                        options: String::from("i"),
                    }
                }},
                { "history": {
                    "$elemMatch": {
                        "comment": {
                            "$regex": Regex {
                                pattern,
                                options: String::from("i"),
                            }
                        }
                    }
                }}
            ]
        };

        let mut cursor = self.store.find(query).session(&mut *session).await?;
        let mut requests = Vec::new();
        while let Some(request) = cursor.next(&mut *session).await {
            requests.push(request?);
        }
        Ok(requests)
    }

    pub async fn get(&self, session: &mut Session, id: ObjectId) -> Result<Option<Request>, Error> {
        let request = self
            .store
            .find_one(doc! { "_id": id })
            .session(&mut *session)
            .await?;
        Ok(request)
    }

    pub async fn update(&self, session: &mut Session, request: &Request) -> Result<(), Error> {
        self.store
            .replace_one(doc! { "_id": request.id }, request)
            .session(&mut *session)
            .await?;
        Ok(())
    }

    pub async fn create(&self, session: &mut Session, request: Request) -> Result<(), Error> {
        self.store
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
            .store
            .find_one(doc! { "phone": phone })
            .session(&mut *session)
            .await?;
        Ok(request)
    }

    pub async fn find_range(
        &self,
        session: &mut Session,
        from: Option<DateTime<Local>>,
        to: Option<DateTime<Local>>,
    ) -> Result<SessionCursor<Request>, Error> {
        let mut query = doc! {};
        if let Some(from) = from {
            query.insert("created", doc! { "$gte": from });
        }
        if let Some(to) = to {
            query.insert("created", doc! { "$lt": to });
        }
        let cursor = self.store.find(query).session(&mut *session).await?;
        Ok(cursor)
    }

    pub async fn to_notify(&self, session: &mut Session) -> Result<Vec<Request>, Error> {
        let now = Utc::now();
        let mut cursor = self
            .store
            .find(doc! {
                "remind_later.date_time": {
                    "$lt": now,
                }
            })
            .session(&mut *session)
            .await?;

        let mut requests = Vec::new();
        while let Some(request) = cursor.next(&mut *session).await {
            requests.push(request?);
        }
        Ok(requests)
    }

    pub async fn get_all_page(
        &self,
        session: &mut Session,
        limit: i64,
        offset: u64,
    ) -> Result<Vec<Request>, Error> {
        let mut cursor = self
            .store
            .find(doc! {})
            .session(&mut *session)
            .skip(offset)
            .limit(limit)
            .sort(doc! {"created": -1})
            .await?;

        let mut requests = Vec::new();
        while let Some(request) = cursor.next(&mut *session).await {
            requests.push(request?);
        }
        Ok(requests)
    }
}
