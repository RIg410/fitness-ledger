use std::ops::Deref;

use chrono::{DateTime, Local, Utc};
use eyre::{Error, Result};
use model::{
    day::Day,
    ids::DayId,
    rights::Rule,
    slot::Slot,
    training::{Training, TrainingStatus},
};
use mongodb::{bson::oid::ObjectId, ClientSession, SessionCursor};
use storage::{calendar::CalendarStore, user::UserStore};
use thiserror::Error;
use tx_macro::tx;

use crate::programs::Programs;

#[derive(Clone)]
pub struct Calendar {
    calendar: CalendarStore,
    users: UserStore,
    programs: Programs,
}

impl Calendar {
    pub(crate) fn new(calendar: CalendarStore, users: UserStore, programs: Programs) -> Self {
        Calendar {
            calendar,
            users,
            programs,
        }
    }

    pub async fn get_day(&self, session: &mut ClientSession, day: DayId) -> Result<Day> {
        self.calendar.get_day(session, day).await
    }

    pub async fn week_days_after(
        &self,
        session: &mut ClientSession,
        day: DayId,
    ) -> Result<SessionCursor<Day>> {
        self.calendar.week_days_after(session, day).await
    }

    pub async fn get_training_by_start_at(
        &self,
        session: &mut ClientSession,
        id: DateTime<Local>,
    ) -> Result<Option<Training>, Error> {
        let day = self.get_day(session, DayId::from(id)).await?;
        Ok(day
            .training
            .iter()
            .find(|slot| slot.start_at == id)
            .map(|slot| slot.clone()))
    }

    #[tx]
    pub async fn cancel_training(
        &self,
        session: &mut ClientSession,
        training: &Training,
    ) -> Result<()> {
        let mut day = self.get_day(session, training.day_id()).await?;
        let training = day.training.iter_mut().find(|slot| slot.id == training.id);

        if let Some(training) = training {
            self.calendar
                .set_cancel_flag(session, training.start_at, true)
                .await
        } else {
            Err(eyre::eyre!("Training not found"))
        }
    }

    #[tx]
    pub async fn restore_training(
        &self,
        session: &mut ClientSession,
        training: &Training,
    ) -> Result<()> {
        let mut day = self.get_day(session, training.day_id()).await?;
        let training = day.training.iter_mut().find(|slot| slot.id == training.id);

        if let Some(training) = training {
            if training.status(Local::now()) != TrainingStatus::Cancelled {
                return Err(eyre::eyre!("Training is not cancelled"));
            }
            self.calendar
                .set_cancel_flag(session, training.start_at, false)
                .await
        } else {
            return Err(eyre::eyre!("Training not found"));
        }
    }

    #[tx]
    pub async fn delete_training(
        &self,
        session: &mut ClientSession,
        training: &Training,
        all: bool,
    ) -> Result<()> {
        if let Some(training) = self
            .get_training_by_start_at(session, training.get_slot().start_at())
            .await?
        {
            if !training.clients.is_empty() {
                return Err(eyre::eyre!("Training has clients"));
            }

            self.calendar
                .delete_training(session, training.start_at)
                .await?;

            let day_id = DayId::from(training.get_slot().start_at());
            if all {
                let mut cursor = self.calendar.week_days_after(session, day_id).await?;
                while let Some(day) = cursor.next(session).await {
                    let day = day?;
                    let training = day.training.iter().find(|slot| slot.id == training.id);
                    if let Some(training) = training {
                        self.calendar
                            .delete_training(session, training.start_at)
                            .await?;
                    }
                }
            }
        } else {
            return Err(eyre::eyre!("Training not found:{}", training.id));
        }

        Ok(())
    }

    pub async fn get_users_trainings(
        &self,
        session: &mut ClientSession,
        client: ObjectId,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<Training>> {
        self.calendar
            .find_trainings(session, client, limit, offset)
            .await
    }

    #[tx]
    pub async fn schedule(
        &self,
        session: &mut ClientSession,
        object_id: ObjectId,
        start_at: DateTime<Local>,
        instructor: i64,
        is_one_time: bool,
    ) -> Result<(), ScheduleError> {
        let program = self
            .programs
            .get_by_id(session, object_id)
            .await?
            .ok_or(ScheduleError::ProgramNotFound)?;

        let instructor = self
            .users
            .get_by_tg_id(session, instructor)
            .await?
            .ok_or(ScheduleError::InstructorNotFound)?;
        if !instructor.rights.has_rule(Rule::Train) {
            return Err(ScheduleError::InstructorHasNoRights);
        }
        let collision = self
            .check_time_slot(session, program.id, start_at, is_one_time)
            .await?;
        if let Some(collision) = collision {
            return Err(ScheduleError::TimeSlotCollision(collision));
        }

        let day_id = DayId::from(start_at);
        let slot = Slot::new(start_at.with_timezone(&Utc), program.duration_min);
        let day = self.get_day(session, day_id).await?;
        for training in day.training {
            if training.get_slot().has_conflict(&slot) {
                return Err(ScheduleError::TimeSlotCollision(TimeSlotCollision(
                    training,
                )));
            }
        }
        let mut training = Training::with_program(program, start_at, instructor.id, is_one_time);
        if !training.status(Local::now()).can_sign_in() {
            return Err(ScheduleError::TooCloseToStart);
        }

        self.calendar.add_training(session, &training).await?;

        if !is_one_time {
            let mut cursor = self.calendar.week_days_after(session, day_id).await?;
            while let Some(day) = cursor.next(session).await {
                let day = day?;
                training = Training::with_day_and_training(day.day_id(), training);
                let slot = slot.with_day(day.day_id());
                for training in day.training {
                    if training.get_slot().has_conflict(&slot) {
                        return Err(ScheduleError::TimeSlotCollision(TimeSlotCollision(
                            training,
                        )));
                    }
                }
                self.calendar.add_training(session, &training).await?;
            }
        }

        Ok(())
    }

    pub async fn check_time_slot(
        &self,
        session: &mut ClientSession,
        program_id: ObjectId,
        start_at: DateTime<Local>,
        is_one_time: bool,
    ) -> Result<Option<TimeSlotCollision>> {
        let program = self
            .programs
            .get_by_id(session, program_id)
            .await?
            .ok_or_else(|| eyre::eyre!("Program not found:{}", program_id))?;

        let day_id = DayId::from(start_at);
        let slot = Slot::new(start_at.with_timezone(&Utc), program.duration_min);
        let day = self.get_day(session, day_id).await?;
        for training in day.training {
            if training.get_slot().has_conflict(&slot) {
                return Ok(Some(TimeSlotCollision(training)));
            }
        }

        if !is_one_time {
            let mut cursor = self.calendar.week_days_after(session, day_id).await?;
            while let Some(day) = cursor.next(session).await {
                let day = day?;
                let slot = slot.with_day(day.day_id());
                for training in day.training {
                    if training.get_slot().has_conflict(&slot) {
                        return Ok(Some(TimeSlotCollision(training)));
                    }
                }
            }
        }

        Ok(None)
    }

    pub async fn days_for_process(
        &self,
        session: &mut ClientSession,
    ) -> Result<SessionCursor<Day>> {
        self.calendar.days_to_process(session).await
    }
}

impl Calendar {
    pub(crate) async fn sign_up(
        &self,
        session: &mut ClientSession,
        start_at: DateTime<Utc>,
        user_id: ObjectId,
    ) -> Result<()> {
        self.calendar.sign_up(session, start_at, user_id).await
    }

    pub(crate) async fn sign_out(
        &self,
        session: &mut ClientSession,
        start_at: DateTime<Utc>,
        user_id: ObjectId,
    ) -> Result<()> {
        self.calendar.sign_out(session, start_at, user_id).await
    }

    pub(crate) async fn finalized(
        &self,
        session: &mut ClientSession,
        start_at: DateTime<Utc>,
    ) -> Result<()> {
        self.calendar.finalized(session, start_at).await
    }
}

#[derive(Debug)]
pub struct TimeSlotCollision(Training);

impl Deref for TimeSlotCollision {
    type Target = Training;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Error)]
pub enum ScheduleError {
    #[error("Program not found")]
    ProgramNotFound,
    #[error("Instructor not found")]
    InstructorNotFound,
    #[error("Instructor has no rights")]
    InstructorHasNoRights,
    #[error("Too close to start")]
    TooCloseToStart,
    #[error("Time slot collision:{0:?}")]
    TimeSlotCollision(TimeSlotCollision),
    #[error("Common error:{0}")]
    Common(#[from] eyre::Error),
}

impl From<TimeSlotCollision> for ScheduleError {
    fn from(e: TimeSlotCollision) -> Self {
        ScheduleError::TimeSlotCollision(e)
    }
}

impl From<mongodb::error::Error> for ScheduleError {
    fn from(e: mongodb::error::Error) -> Self {
        ScheduleError::Common(e.into())
    }
}

#[derive(Debug, Error)]
pub enum SignOutError {
    #[error("Training not found")]
    TrainingNotFound,
    #[error("Training is not open to sign out")]
    TrainingNotOpenToSignOut,
    #[error("Client not signed up")]
    ClientNotSignedUp,
    #[error("Common error:{0}")]
    Common(#[from] eyre::Error),
    #[error("Not enough reserved balance")]
    NotEnoughReservedBalance,
    #[error("User not found")]
    UserNotFound,
}

impl From<mongodb::error::Error> for SignOutError {
    fn from(e: mongodb::error::Error) -> Self {
        SignOutError::Common(e.into())
    }
}
