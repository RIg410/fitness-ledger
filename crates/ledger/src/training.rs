use crate::Ledger;
use eyre::Error;
use model::{session::Session, training::Training};
use mongodb::bson::oid::ObjectId;
use tx_macro::tx;

impl Ledger {
    #[tx]
    pub async fn cancel_training(
        &self,
        session: &mut Session,
        training: &Training,
    ) -> Result<Vec<ObjectId>, Error> {
        let training = self.calendar.cancel_training(session, training).await?;
        Ok(training.clients)
    }
}
