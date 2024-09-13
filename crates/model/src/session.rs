use std::ops::{Deref, DerefMut};

use bson::oid::ObjectId;
use mongodb::ClientSession;

pub struct Session {
    client_session: ClientSession,
    actor: ObjectId,
}

impl Session {
    pub fn new(client_session: ClientSession, actor: ObjectId) -> Self {
        Session {
            client_session,
            actor,
        }
    }

    pub fn actor(&self) -> ObjectId {
        self.actor
    }

    pub fn set_actor(&mut self, actor: ObjectId) {
        self.actor = actor;
    }
}

impl Deref for Session {
    type Target = ClientSession;

    fn deref(&self) -> &Self::Target {
        &self.client_session
    }
}

impl DerefMut for Session {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.client_session
    }
}

impl<'a> From<&'a mut Session> for &'a mut ClientSession {
    fn from(session: &'a mut Session) -> &'a mut ClientSession {
        &mut session.client_session
    }
}
