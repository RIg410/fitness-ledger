use crate::Ledger;
use chrono::{DateTime, Local};
use model::{
    decimal::Decimal,
    errors::LedgerError,
    session::Session,
    training::{Training, TrainingId},
    user::family::FindFor,
};
use mongodb::bson::oid::ObjectId;
use tx_macro::tx;

impl Ledger {
    #[tx]
    pub async fn cancel_training(
        &self,
        session: &mut Session,
        training: &Training,
    ) -> Result<Vec<ObjectId>, LedgerError> {
        for client in &training.clients {
            self.sign_out_tx_less(session, training, *client, false)
                .await?;
        }
        let training = self.calendar.cancel_training(session, training).await?;
        if training.tp.is_personal() {
            self.calendar
                .delete_training_txless(session, training.id(), false)
                .await?;
        }
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
    ) -> Result<(), LedgerError> {
        let id = self
            .calendar
            .schedule_personal_training(session, client, instructor, start_at, duration_min, room)
            .await?;
        self.sign_up_txless(session, id, client, true).await?;
        Ok(())
    }

    #[tx]
    pub async fn sign_up(
        &self,
        session: &mut Session,
        id: TrainingId,
        client: ObjectId,
        forced: bool,
    ) -> Result<(), LedgerError> {
        let training = self
            .calendar
            .get_training_by_id(session, id)
            .await?
            .ok_or_else(|| LedgerError::TrainingNotFound(id))?;
        let status = training.status(Local::now());
        if !forced && !status.can_sign_in() {
            return Err(LedgerError::TrainingNotOpenToSignUp(id, status));
        }

        if training.is_processed {
            return Err(LedgerError::TrainingNotOpenToSignUp(id, status));
        }

        if training.clients.contains(&client) {
            return Err(LedgerError::ClientAlreadySignedUp(client, id));
        }

        if training.clients.len() as u32 >= training.capacity {
            return Err(LedgerError::TrainingIsFull(id));
        }

        let mut user = self
            .users
            .get(session, client)
            .await?
            .ok_or_else(|| LedgerError::ClientNotFound(client))?;
        let user_id = user.id;

        self.users.resolve_family(session, &mut user).await?;
        let mut payer = user.payer_mut()?;

        if training.tp.is_not_free() {
            let subscription = payer
                .find_subscription(FindFor::Lock, &training)
                .ok_or_else(|| LedgerError::NotEnoughBalance(user_id))?;

            if !subscription.lock_balance() {
                return Err(LedgerError::NotEnoughBalance(user_id));
            }
            self.users.update(session, &mut payer).await?;
        }

        self.calendar
            .sign_up(session, training.id(), client)
            .await?;
        self.history
            .sign_up(
                session,
                user_id,
                training.get_slot().start_at(),
                training.name,
            )
            .await?;
        Ok(())
    }

    #[tx]
    pub async fn sign_out(
        &self,
        session: &mut Session,
        id: TrainingId,
        client: ObjectId,
        forced: bool,
    ) -> Result<(), LedgerError> {
        let training = self
            .calendar
            .get_training_by_id(session, id)
            .await?
            .ok_or_else(|| LedgerError::TrainingNotFound(id))?;
        self.sign_out_tx_less(session, &training, client, forced)
            .await?;

        if training.tp.is_personal() {
            self.calendar
                .delete_training_txless(session, training.id(), false)
                .await?;
        }
        Ok(())
    }

    pub(crate) async fn sign_out_tx_less(
        &self,
        session: &mut Session,
        training: &Training,
        client: ObjectId,
        forced: bool,
    ) -> Result<(), LedgerError> {
        let status = training.status(Local::now());
        if !forced && !status.can_sign_out() {
            return Err(LedgerError::TrainingNotOpenToSignOut(training.id()));
        }

        if training.is_processed {
            return Err(LedgerError::TrainingNotOpenToSignOut(training.id()));
        }

        if !training.clients.contains(&client) {
            return Err(LedgerError::ClientNotSignedUp(client, training.id()));
        }

        let mut user = self
            .users
            .get(session, client)
            .await?
            .ok_or_else(|| LedgerError::UserNotFound(client))?;
        self.users.resolve_family(session, &mut user).await?;

        let user_id = user.id;
        let mut payer = user.payer_mut()?;

        if training.tp.is_not_free() {
            let sub = payer
                .find_subscription(FindFor::Unlock, training)
                .ok_or_else(|| LedgerError::NotEnoughReservedBalance(client))?;

            if !sub.unlock_balance() {
                return Err(LedgerError::NotEnoughReservedBalance(client));
            }
            self.users.update(session, &mut payer).await?;
        }

        self.calendar
            .sign_out(session, training.id(), client)
            .await?;
        self.history
            .sign_out(
                session,
                user_id,
                training.get_slot().start_at(),
                training.name.clone(),
            )
            .await?;
        Ok(())
    }
}
