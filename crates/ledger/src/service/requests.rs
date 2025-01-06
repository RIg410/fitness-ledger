use std::{ops::Deref, sync::Arc};

use bson::oid::ObjectId;
use chrono::Utc;
use eyre::Error;
use eyre::Result;
use model::errors::LedgerError;
use model::request::Request;
use model::request::RequestHistoryRow;
use model::user::sanitize_phone;
use model::{request::RemindLater, session::Session, statistics::marketing::ComeFrom};
use storage::requests::RequestStore;
use tx_macro::tx;

use super::users::Users;

#[derive(Clone)]
pub struct Requests {
    requests: Arc<RequestStore>,
    users: Users,
}

impl Requests {
    pub fn new(store: Arc<RequestStore>, users: Users) -> Self {
        Requests {
            requests: store,
            users,
        }
    }

    #[tx]
    pub async fn update_come_from(
        &self,
        session: &mut Session,
        id: ObjectId,
        come_from: ComeFrom,
        comment: String,
    ) -> Result<(), LedgerError> {
        if let Some(mut request) = self.requests.get(session, id).await? {
            request.history.push(RequestHistoryRow {
                comment: request.comment.clone(),
                date_time: request.modified,
            });
            request.modified = Utc::now();
            request.comment = comment;
            request.come_from = come_from;
            self.requests.update(session, &request).await?;

            let user = self
                .users
                .get_by_phone(session, &sanitize_phone(&request.phone))
                .await?;
            if let Some(user) = user {
                self.users
                    .update_come_from(session, user.id, come_from)
                    .await?;
            }
        } else {
            return Err(LedgerError::RequestNotFound { id });
        }
        Ok(())
    }

    #[tx]
    pub async fn add_comment(
        &self,
        session: &mut Session,
        id: ObjectId,
        comment: String,
    ) -> Result<(), LedgerError> {
        if let Some(mut request) = self.requests.get(session, id).await? {
            request.history.push(RequestHistoryRow {
                comment: request.comment.clone(),
                date_time: request.modified,
            });
            request.modified = Utc::now();
            request.comment = comment;
            self.requests.update(session, &request).await?;
        } else {
            return Err(LedgerError::RequestNotFound { id });
        }
        Ok(())
    }

    #[tx]
    pub async fn add_notification(
        &self,
        session: &mut Session,
        id: ObjectId,
        remember_later: Option<RemindLater>,
    ) -> Result<(), LedgerError> {
        if let Some(mut request) = self.requests.get(session, id).await? {
            request.remind_later = remember_later;
            self.requests.update(session, &request).await?;
        } else {
            return Err(LedgerError::RequestNotFound { id });
        }
        Ok(())
    }

    #[tx]
    pub async fn create_request(
        &self,
        session: &mut Session,
        phone: String,
        come_from: ComeFrom,
        comment: String,
        first_name: Option<String>,
        last_name: Option<String>,
        remember_later: Option<RemindLater>,
    ) -> Result<()> {
        let phone = sanitize_phone(&phone);
        let user = self.users.get_by_phone(session, &phone).await?;
        if let Some(user) = user {
            self.users
                .update_come_from(session, user.id, come_from)
                .await?;
        }
        if let Some(mut request) = self.requests.get_by_phone(session, &phone).await? {
            request.remind_later = remember_later;
            request.history.push(RequestHistoryRow {
                comment: request.comment.clone(),
                date_time: request.modified,
            });
            request.modified = Utc::now();
            request.comment = comment;
            request.come_from = come_from;
            self.requests.update(session, &request).await?;
        } else {
            self.requests
                .create(
                    session,
                    Request::new(
                        phone,
                        comment,
                        come_from,
                        first_name,
                        last_name,
                        remember_later,
                    ),
                )
                .await?;
        }
        Ok(())
    }

    pub async fn come_from(&self, session: &mut Session, phone: &str) -> Result<ComeFrom, Error> {
        let phone = model::user::sanitize_phone(phone);
        self.requests
            .get_by_phone(session, &phone)
            .await
            .map(|r| r.map(|r| r.come_from).unwrap_or_default())
    }
}

impl Deref for Requests {
    type Target = RequestStore;

    fn deref(&self) -> &Self::Target {
        &self.requests
    }
}
