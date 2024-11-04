use crate::bot::TgBot;
use ledger::Ledger;
use model::{rights::Rule, session::Session, user::User};
use mongodb::bson::oid::ObjectId;
use std::{ops::{Deref, DerefMut}, sync::Arc};

pub struct Context {
    pub bot: TgBot,
    pub me: User,
    pub ledger: Arc<Ledger>,
    pub session: Session,
    pub is_real_user: bool,
}

impl Context {
    pub fn new(
        bot: TgBot,
        me: User,
        ledger: Arc<Ledger>,
        session: Session,
        is_real_user: bool,
    ) -> Context {
        Context {
            bot,
            me,
            ledger,
            session,
            is_real_user,
        }
    }

    pub fn is_me(&self, id: ObjectId) -> bool {
        self.me.id == id
    }

    pub fn is_couch(&self) -> bool {
        self.me.couch.is_some()
    }

    pub fn is_active(&self) -> bool {
        self.me.is_active
    }

    pub fn is_admin(&self) -> bool {
        self.me.rights.is_admin()
    }

    pub fn has_right(&self, rule: Rule) -> bool {
        self.me.rights.has_rule(rule)
    }

    pub fn ensure(&self, rule: Rule) -> Result<(), eyre::Error> {
        self.me.rights.ensure(rule)
    }

    pub async fn reload_user(&mut self) -> Result<(), eyre::Error> {
        let user = self
            .ledger
            .users
            .get(&mut self.session, self.me.id)
            .await?
            .ok_or_else(|| eyre::eyre!("Failed to load existing user:{}", self.me.id))?;

        self.me = user;
        Ok(())
    }
}

impl Deref for Context {
    type Target = TgBot;

    fn deref(&self) -> &Self::Target {
        &self.bot
    }
}

impl DerefMut for Context {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.bot
    }
}
