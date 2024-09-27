use crate::history::History;
use eyre::Error;
use model::{program::Program, session::Session};
use mongodb::bson::oid::ObjectId;
use std::ops::Deref;
use storage::program::ProgramStore;
use tx_macro::tx;

#[derive(Clone)]
pub struct Programs {
    store: ProgramStore,
    _logs: History,
}

impl Programs {
    pub fn new(store: ProgramStore, logs: History) -> Self {
        Programs { store, _logs: logs }
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
        // self.logs.create_program(session, proto).await;
        Ok(())
    }
}

impl Deref for Programs {
    type Target = ProgramStore;

    fn deref(&self) -> &Self::Target {
        &self.store
    }
}
