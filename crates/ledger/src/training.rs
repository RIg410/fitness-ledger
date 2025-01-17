use crate::{
    service::calendar::{ScheduleError, SignOutError},
    Ledger, SignUpError,
};
use chrono::{DateTime, Local};
use eyre::Error;
use model::{
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
    ) -> Result<Vec<ObjectId>, Error> {
        for client in &training.clients {
            self.sign_out_tx_less(session, training, *client, false)
                .await?;
        }
        let training = self.calendar.cancel_training(session, training).await?;
        if training.tp.is_personal() {
            self.calendar
                .delete_training_txless(session, training.id(), false)
                .await
                .map_err(|_| SignOutError::TrainingNotFound)?;
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

    #[tx]
    pub async fn sign_up(
        &self,
        session: &mut Session,
        id: TrainingId,
        client: ObjectId,
        forced: bool,
    ) -> Result<(), SignUpError> {
        let training = self
            .calendar
            .get_training_by_id(session, id)
            .await?
            .ok_or_else(|| SignUpError::TrainingNotFound)?;
        let status = training.status(Local::now());
        if !forced && !status.can_sign_in() {
            return Err(SignUpError::TrainingNotOpenToSignUp(status));
        }

        if training.is_processed {
            return Err(SignUpError::TrainingNotOpenToSignUp(status));
        }

        if training.clients.contains(&client) {
            return Err(SignUpError::ClientAlreadySignedUp);
        }

        if training.clients.len() as u32 >= training.capacity {
            return Err(SignUpError::TrainingIsFull);
        }

        let mut user = self
            .users
            .get(session, client)
            .await?
            .ok_or_else(|| SignUpError::UserNotFound)?;
        let user_id = user.id;

        if user.employee.is_some() {
            return Err(SignUpError::UserIsCouch);
        }

        self.users.resolve_family(session, &mut user).await?;
        let mut payer = user.payer_mut()?;

        if training.tp.is_not_free() {
            let subscription = payer
                .find_subscription(FindFor::Lock, &training)
                .ok_or_else(|| SignUpError::NotEnoughBalance)?;

            if !subscription.lock_balance() {
                return Err(SignUpError::NotEnoughBalance);
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
    ) -> Result<(), SignOutError> {
        let training = self
            .calendar
            .get_training_by_id(session, id)
            .await?
            .ok_or_else(|| SignOutError::TrainingNotFound)?;
        self.sign_out_tx_less(session, &training, client, forced)
            .await?;

        if training.tp.is_personal() {
            self.calendar
                .delete_training_txless(session, training.id(), false)
                .await
                .map_err(|_| SignOutError::TrainingNotFound)?;
        }
        Ok(())
    }

    pub(crate) async fn sign_out_tx_less(
        &self,
        session: &mut Session,
        training: &Training,
        client: ObjectId,
        forced: bool,
    ) -> Result<(), SignOutError> {
        let status = training.status(Local::now());
        if !forced && !status.can_sign_out() {
            return Err(SignOutError::TrainingNotOpenToSignOut);
        }

        if training.is_processed {
            return Err(SignOutError::TrainingNotOpenToSignOut);
        }

        if !training.clients.contains(&client) {
            return Err(SignOutError::ClientNotSignedUp);
        }

        let mut user = self
            .users
            .get(session, client)
            .await?
            .ok_or_else(|| SignOutError::UserNotFound)?;
        self.users.resolve_family(session, &mut user).await?;

        let user_id = user.id;
        let mut payer = user.payer_mut()?;

        if training.tp.is_not_free() {
            let sub = payer
                .find_subscription(FindFor::Unlock, training)
                .ok_or_else(|| SignOutError::NotEnoughReservedBalance)?;

            if !sub.unlock_balance() {
                return Err(SignOutError::NotEnoughReservedBalance);
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
