use chrono::{DateTime, Datelike, Local};
use eyre::Error;
use futures_util::StreamExt as _;
use mongodb::bson::oid::ObjectId;
use storage::training::model::{Training, TrainingProto};

use crate::Ledger;

// todo make transactional
impl Ledger {
    pub async fn find_trainings(&self, query: Option<&str>) -> Result<Vec<TrainingProto>, Error> {
        Ok(self.training.find(query).await?)
    }

    pub async fn get_training_by_name(&self, name: &str) -> Result<Option<TrainingProto>, Error> {
        Ok(self.training.get_by_name(name).await?)
    }

    pub async fn get_training_by_id(&self, id: ObjectId) -> Result<Option<TrainingProto>, Error> {
        Ok(self.training.get_by_id(id).await?)
    }

    pub async fn create_training_proto(&self, proto: &TrainingProto) -> Result<(), Error> {
        let training = self.get_training_by_name(&proto.name).await?;
        if training.is_some() {
            return Err(eyre::eyre!("Training with this name already exists"));
        }

        Ok(self.training.insert(proto).await?)
    }

    pub async fn add_training(
        &self,
        proto_id: ObjectId,
        start_at: DateTime<Local>,
        instructor: i64,
        is_one_time: bool,
    ) -> Result<(), AddTrainingError> {
        let proto = self
            .get_training_by_id(proto_id)
            .await?
            .ok_or(AddTrainingError::ProtoTrainingNotFound)?;

        let instructor = self
            .users
            .get_by_tg_id(instructor)
            .await?
            .ok_or(AddTrainingError::InstructorNotFound)?;

        if !instructor
            .rights
            .has_rule(storage::user::rights::Rule::Train)
        {
            return Err(AddTrainingError::InstructorHasNoRights);
        }

        let mut week = self.calendar.get_week(Some(start_at)).await?;
        let weekday = start_at.weekday();
        let day = week.get_day_mut(weekday);

        let training = Training {
            id: ObjectId::new(),
            proto_id,
            name: proto.name.clone(),
            description: proto.description.clone(),
            start_at,
            duration_min: proto.duration_min,
            instructor: instructor.id,
            clients: vec![],
            capacity: proto.capacity,
            status: storage::training::model::TrainingStatus::OpenToSignup,
            is_one_time: is_one_time,
        };
        let ok = day.add_training(training.clone());
        if !ok {
            return Err(AddTrainingError::TimeSlotOccupied);
        }
        self.calendar.update_week(week).await?;

        if !is_one_time {
            let mut weeks = self.calendar.week_cursor(start_at).await?;
            while let Some(week) = weeks.next().await {
                let mut training = training.clone();
                let mut week = week?;
                let week_date = week.day_date(weekday);
                let day = week.get_day_mut(weekday);
                training.set_date(week_date)?;
                let ok = day.add_training(training.clone());
                if !ok {
                    return Err(AddTrainingError::TimeSlotOccupied);
                }
                self.calendar.update_week(week).await?;
            }
        }

        Ok(())
    }
}

pub enum AddTrainingError {
    ProtoTrainingNotFound,
    InstructorNotFound,
    InstructorHasNoRights,
    TimeSlotOccupied,
    Common(eyre::Error),
}

impl From<eyre::Error> for AddTrainingError {
    fn from(value: eyre::Error) -> Self {
        AddTrainingError::Common(value)
    }
}

impl From<mongodb::error::Error> for AddTrainingError {
    fn from(value: mongodb::error::Error) -> Self {
        AddTrainingError::Common(value.into())
    }
}
