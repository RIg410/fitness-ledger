use std::ops::Deref;

use eyre::Error;
use model::{session::Session, statistics::marketing::ComeFrom};
use storage::requests::RequestStore;

#[derive(Clone)]
pub struct Requests {
    store: RequestStore,
}

impl Requests {
    pub fn new(store: RequestStore) -> Self {
        Requests { store }
    }

    pub async fn come_from(&self, session: &mut Session, phone: &str) -> Result<ComeFrom, Error> {
        let phone = model::user::sanitize_phone(phone);
        self.store
            .get_by_phone(session, &phone)
            .await
            .map(|r| r.map(|r| r.come_from).unwrap_or_default())
    }
}

impl Deref for Requests {
    type Target = RequestStore;

    fn deref(&self) -> &Self::Target {
        &self.store
    }
}
