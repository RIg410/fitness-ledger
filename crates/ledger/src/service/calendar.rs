use std::{ops::Deref, sync::Arc};

use chrono::{DateTime, Local, Utc};
use eyre::{Error, Result};
use model::{
    ids::DayId,
    session::Session,
    slot::Slot,
    training::{Training, TrainingId, TrainingStatus},
};
use mongodb::bson::oid::ObjectId;
use storage::calendar::CalendarStore;
use thiserror::Error;
use tx_macro::tx;

use super::{programs::Programs, users::Users};

#[derive(Clone)]
pub struct Calendar {
    calendar: Arc<CalendarStore>,
    users: Users,
    programs: Programs,
}

impl Calendar {
    pub(crate) fn new(calendar: Arc<CalendarStore>, users: Users, programs: Programs) -> Self {
        Calendar {
            calendar,
            users,
            programs,
        }
    }

    pub async fn get_training_by_id(
        &self,
        session: &mut Session,
        id: TrainingId,
    ) -> Result<Option<Training>, Error> {
        let day = self.get_day(session, DayId::from(id.start_at)).await?;
        Ok(day.training.iter().find(|slot| slot.id() == id).cloned())
    }

    pub(crate) async fn cancel_training(
        &self,
        session: &mut Session,
        training: &Training,
    ) -> Result<Training> {
        let day = self.get_day(session, training.day_id()).await?;
        let training = day.training.into_iter().find(|slot| slot.id == training.id);

        if let Some(training) = training {
            self.calendar
                .set_cancel_flag(session, training.id(), true)
                .await?;
            Ok(training)
        } else {
            Err(eyre::eyre!("Training not found"))
        }
    }

    #[tx]
    pub async fn restore_training(&self, session: &mut Session, training: &Training) -> Result<()> {
        let mut day = self.get_day(session, training.day_id()).await?;
        let training = day
            .training
            .iter_mut()
            .find(|slot| slot.id() == training.id());

        if let Some(training) = training {
            if training.status(Local::now()) != TrainingStatus::Cancelled {
                return Err(eyre::eyre!("Training is not cancelled"));
            }
            self.calendar
                .set_cancel_flag(session, training.id(), false)
                .await?;
            Ok(())
        } else {
            return Err(eyre::eyre!("Training not found"));
        }
    }

    #[tx]
    pub async fn change_couch(
        &self,
        session: &mut Session,
        id: TrainingId,
        new_couch: ObjectId,
        all: bool,
    ) -> Result<(), Error> {
        if let Some(training) = self.get_training_by_id(session, id).await? {
            self.calendar.change_couch(session, id, new_couch).await?;

            let day_id = DayId::from(training.get_slot().start_at());
            if all {
                let mut cursor = self.calendar.week_days_after(session, day_id).await?;
                while let Some(day) = cursor.next(session).await {
                    let day = day?;
                    let training = day.training.iter().find(|slot| slot.id == training.id);
                    if let Some(training) = training {
                        self.calendar
                            .change_couch(session, training.id(), new_couch)
                            .await?;
                    }
                }
            }
        } else {
            return Err(eyre::eyre!("Training not found:{:?}", id));
        }

        Ok(())
    }

    #[tx]
    pub async fn delete_training(
        &self,
        session: &mut Session,
        id: TrainingId,
        all: bool,
    ) -> Result<()> {
        if let Some(training) = self.get_training_by_id(session, id).await? {
            if !training.clients.is_empty() {
                return Err(eyre::eyre!("Training has clients"));
            }

            self.calendar.delete_training(session, id).await?;

            let day_id = DayId::from(training.get_slot().start_at());
            if all {
                let mut cursor = self.calendar.week_days_after(session, day_id).await?;
                while let Some(day) = cursor.next(session).await {
                    let day = day?;
                    let training = day.training.iter().find(|slot| slot.id == training.id);
                    if let Some(training) = training {
                        if !training.clients.is_empty() {
                            return Err(eyre::eyre!("Training has clients"));
                        }
                        self.calendar.delete_training(session, id).await?;
                    }
                }
            }
        } else {
            return Err(eyre::eyre!("Training not found:{:?}", id));
        }

        Ok(())
    }

    pub(crate) async fn schedule_personal_training(
        &self,
        session: &mut Session,
        client: ObjectId,
        instructor: ObjectId,
        start_at: DateTime<Local>,
        duration_min: u32,
        room: ObjectId,
    ) -> Result<TrainingId, ScheduleError> {
        let instructor = self
            .users
            .get(session, instructor)
            .await?
            .ok_or(ScheduleError::InstructorNotFound)?;
        if !instructor.is_couch() {
            return Err(ScheduleError::InstructorHasNoRights);
        }
        let client = self
            .users
            .get(session, client)
            .await?
            .ok_or(ScheduleError::ClientNotFound)?;

        let slot = Slot::new(start_at.with_timezone(&Utc), duration_min, room);
        let collision = self.check_time_slot(session, slot, true).await?;
        if let Some(collision) = collision {
            return Err(ScheduleError::TimeSlotCollision(collision));
        }

        let name = format!(
            "Инди:{}/{}",
            client.name.first_name, instructor.name.first_name
        );
        let description = instructor
            .employee
            .map(|e| e.description.clone())
            .unwrap_or_default();
        let training = Training::new_personal(
            start_at,
            room,
            instructor.id,
            duration_min,
            name,
            description,
        );

        self.calendar.add_training(session, &training).await?;
        Ok(training.id())
    }

    #[tx]
    pub async fn schedule_group(
        &self,
        session: &mut Session,
        program_id: ObjectId,
        start_at: DateTime<Local>,
        room: ObjectId,
        instructor: ObjectId,
        is_one_time: bool,
    ) -> Result<(), ScheduleError> {
        let program = self
            .programs
            .get_by_id(session, program_id)
            .await?
            .ok_or(ScheduleError::ProgramNotFound)?;

        let instructor = self
            .users
            .get(session, instructor)
            .await?
            .ok_or(ScheduleError::InstructorNotFound)?;
        if !instructor.is_couch() {
            return Err(ScheduleError::InstructorHasNoRights);
        }

        let day_id = DayId::from(start_at);
        let slot = Slot::new(start_at.with_timezone(&Utc), program.duration_min, room);
        let collision = self.check_time_slot(session, slot, is_one_time).await?;
        if let Some(collision) = collision {
            return Err(ScheduleError::TimeSlotCollision(collision));
        }

        let mut training = Training::new_group(program, start_at, instructor.id, is_one_time, room);
        if !training.status(Local::now()).can_sign_in() {
            return Err(ScheduleError::TooCloseToStart);
        }

        self.calendar.add_training(session, &training).await?;

        if !is_one_time {
            let mut cursor = self.calendar.week_days_after(session, day_id).await?;
            while let Some(day) = cursor.next(session).await {
                let day = day?;
                training = Training::with_day_and_training(day.day_id(), training);
                self.calendar.add_training(session, &training).await?;
            }
        }

        Ok(())
    }

    pub async fn check_time_slot(
        &self,
        session: &mut Session,
        slot: Slot,
        is_one_time: bool,
    ) -> Result<Option<TimeSlotCollision>> {
        let day_id = slot.day_id();
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
}

impl Calendar {
    pub(crate) async fn edit_duration(
        &self,
        session: &mut Session,
        program_id: ObjectId,
        duration: u32,
    ) -> Result<()> {
        let mut cursor = self
            .calendar
            .find_with_program_id(session, program_id)
            .await?;
        while let Some(day) = cursor.next(session).await {
            let mut day = day?;
            for training in &mut day.training {
                if training.proto_id == program_id {
                    training.duration_min = duration;
                }
            }

            if day.has_conflict() {
                return Err(eyre::eyre!("Conflicts found"));
            }

            self.calendar
                .update_duration_in_day(session, day.id, program_id, duration)
                .await?;
        }

        Ok(())
    }
}

impl Deref for Calendar {
    type Target = CalendarStore;

    fn deref(&self) -> &Self::Target {
        &self.calendar
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
    #[error("Client not found")]
    ClientNotFound,
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
