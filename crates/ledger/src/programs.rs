use crate::logs::Logs;
use eyre::Error;
use model::{program::Program, session::Session};
use mongodb::bson::oid::ObjectId;
use storage::training::ProgramStore;
use tx_macro::tx;

#[derive(Clone)]
pub struct Programs {
    store: ProgramStore,
    logs: Logs,
}

impl Programs {
    pub fn new(store: ProgramStore, logs: Logs) -> Self {
        Programs { store, logs }
    }

    pub async fn find(
        &self,
        session: &mut Session,
        query: Option<&str>,
    ) -> Result<Vec<Program>, Error> {
        self.store.find(session, query).await
    }

    pub async fn get_by_name(
        &self,
        session: &mut Session,
        name: &str,
    ) -> Result<Option<Program>, Error> {
        self.store.get_by_name(session, name).await
    }

    pub async fn get_by_id(
        &self,
        session: &mut Session,
        id: ObjectId,
    ) -> Result<Option<Program>, Error> {
        self.store.get_by_id(session, id).await
    }

    pub async fn get_all(&self, session: &mut Session) -> Result<Vec<Program>, Error> {
        self.store.get_all(session).await
    }

    #[tx]
    pub async fn create(
        &self,
        session: &mut Session,
        name: String,
        description: String,
        duration_min: u32,
        capacity: u32,
    ) -> Result<(), Error> {
        let proto = Program {
            id: ObjectId::new(),
            name,
            description,
            duration_min,
            capacity,
            version: 0,
        };
        let training = self.get_by_name(session, &proto.name).await?;
        if training.is_some() {
            return Err(eyre::eyre!("Training with this name already exists"));
        }

        self.store.insert(session, &proto).await?;
        self.logs.create_program(session, proto).await;
        Ok(())
    }
}
