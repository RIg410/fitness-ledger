use chrono::{DateTime, Local, Utc};
use eyre::Error;
use futures_util::StreamExt as _;
use model::{
    ids::DayId,
    proto::TrainingProto,
    rights::Rule,
    training::{Training, TrainingStatus},
};
use mongodb::bson::oid::ObjectId;
use storage::training::{self};

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

    pub async fn get_all_trainings(&self) -> Result<Vec<TrainingProto>, Error> {
        Ok(self.training.get_all().await?)
    }

    pub async fn create_training_proto(&self, proto: &TrainingProto) -> Result<(), Error> {
        let training = self.get_training_by_name(&proto.name).await?;
        if training.is_some() {
            return Err(eyre::eyre!("Training with this name already exists"));
        }

        Ok(self.training.insert(proto).await?)
    }

    pub async fn delete_training(&self, to_remove: &Training, all: bool) -> Result<(), Error> {
        let day_id = DayId::from(to_remove.start_at);
        let mut day = self.calendar.get_day(day_id).await?;
        day.training.retain(|t| t.id != to_remove.id);
        self.calendar.update_day(&day).await?;
        if all {
            let mut day = self.calendar.cursor(day_id, day_id.week_day()).await?;
            while let Some(day) = day.next().await {
                let mut day = day?;
                let start_at = to_remove.start_at_on(day.day_id());
                day.training
                    .retain(|t| t.start_at != start_at && t.proto_id != to_remove.proto_id);
                self.calendar.update_day(&day).await?;
            }
        }

        Ok(())
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

        if !instructor.rights.has_rule(Rule::Train) {
            return Err(AddTrainingError::InstructorHasNoRights);
        }
        let day_id = DayId::from(start_at);
        let mut day = self.calendar.get_day(day_id).await?;

        let training = Training {
            id: ObjectId::new(),
            proto_id,
            name: proto.name.clone(),
            description: proto.description.clone(),
            start_at: start_at.with_timezone(&Utc),
            duration_min: proto.duration_min,
            instructor: instructor.id,
            clients: vec![],
            capacity: proto.capacity,
            status: TrainingStatus::OpenToSignup,
            is_one_time: is_one_time,
        };
        let ok = day.add_training(training.clone());
        if !ok {
            return Err(AddTrainingError::TimeSlotOccupied);
        }
        self.calendar.update_day(&day).await?;

        if !is_one_time {
            let mut day = self.calendar.cursor(day_id, day_id.week_day()).await?;
            while let Some(day) = day.next().await {
                let mut day = day?;
                let training = training.clone().change_date(day.day_id());
                let ok = day.add_training(training);
                if !ok {
                    return Err(AddTrainingError::TimeSlotOccupied);
                }
                self.calendar.update_day(&day).await?;
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
