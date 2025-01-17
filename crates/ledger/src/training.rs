use crate::{service::calendar::ScheduleError, Ledger};
use chrono::{DateTime, Local};
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
        for client in &training.clients {
            self.sign_out_tx_less(session, training, *client, false)
                .await?;
        }
        let training = self.calendar.cancel_training(session, training).await?;
        Ok(training.clients)
    }

    #[tx]
    pub async fn schedule_personal_training(
        &self,
        session: &mut Session,
        client: ObjectId,
        instructor: ObjectId,
        start_at: DateTime<Local>,
        duration_min: u32,
        room: ObjectId,
    ) -> Result<(), ScheduleError> {
        let id = self
            .calendar
            .schedule_personal_training(session, client, instructor, start_at, duration_min, room)
            .await?;
        self.sign_up_txless(session, id, client, true)
            .await
            .unwrap();

        Ok(())
    }
}
