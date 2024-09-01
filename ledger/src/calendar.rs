use crate::MAX_WEEKS;
use chrono::{DateTime, Datelike as _, Local};
use eyre::{Error, Result};
use mongodb::bson::oid::ObjectId;
use storage::calendar::model::{Day, Week};
use storage::calendar::{day_id, CalendarStore};
use storage::training::model::{Training, TrainingStatus};

pub struct Calendar {
    calendar: CalendarStore,
}

impl Calendar {
    pub(crate) fn new(calendar: CalendarStore) -> Self {
        Calendar { calendar }
    }

    pub async fn get_week(&self, date: Option<DateTime<Local>>) -> Result<Week> {
        let date = date.unwrap_or_else(|| chrono::Local::now());
        if !self.has_week(date) {
            return Err(eyre::eyre!("Week is too far in the future"));
        }

        self.calendar.get_week(date).await
    }

    pub fn has_week(&self, id: DateTime<Local>) -> bool {
        chrono::Local::now() + chrono::Duration::days(7 * MAX_WEEKS as i64) >= id
    }

    pub fn has_next_week(&self, week: &Week) -> bool {
        self.has_week(week.id + chrono::Duration::days(7))
    }

    pub fn has_prev_week(&self, week: &Week) -> bool {
        week.id - chrono::Duration::days(1) >= day_id(chrono::Local::now()).unwrap_or_default()
    }

    pub async fn get_training_by_date(
        &self,
        id: DateTime<Local>,
    ) -> Result<Option<Training>, Error> {
        let week = self.get_week(Some(id)).await?;
        let day = week.get_day(id.weekday());
        Ok(day
            .training
            .iter()
            .find(|slot| slot.is_training_time(id))
            .map(|slot| slot.clone()))
    }

    pub async fn cancel_training(&self, training: &Training) -> Result<()> {
        let mut week = self.get_week(Some(training.start_at)).await?;
        let day = week.get_day_mut(training.start_at.weekday());

        let training = day.training.iter_mut().find(|slot| slot.id == training.id);

        if let Some(training) = training {
            training.status = TrainingStatus::Cancelled;
        } else {
            return Err(eyre::eyre!("Training not found"));
        }
        self.update_week(week).await
    }

    pub async fn uncancel_training(&self, training: &Training) -> Result<()> {
        let mut week = self.get_week(Some(training.start_at)).await?;
        let day = week.get_day_mut(training.start_at.weekday());

        let training = day.training.iter_mut().find(|slot| slot.id == training.id);

        if let Some(training) = training {
            if training.status != TrainingStatus::Cancelled {
                return Err(eyre::eyre!("Training is not cancelled"));
            }
            training.status = TrainingStatus::OpenToSignup;
        } else {
            return Err(eyre::eyre!("Training not found"));
        }
        self.update_week(week).await
    }

    pub async fn sign_up_for_training(&self, training: &Training, client: ObjectId) -> Result<()> {
        let mut week = self.get_week(Some(training.start_at)).await?;
        let day = week.get_day_mut(training.start_at.weekday());

        let training = day.training.iter_mut().find(|slot| slot.id == training.id);

        if let Some(training) = training {
            if training.clients.len() >= training.capacity as usize {
                return Err(eyre::eyre!("Training is full"));
            }
            if training.status != TrainingStatus::OpenToSignup {
                return Err(eyre::eyre!("Training is not open to sign up"));
            }
            if training.clients.contains(&client) {
                return Err(eyre::eyre!("Client already signed up"));
            }
            if training.is_full() {
                return Err(eyre::eyre!("Training is full"));
            }
            training.clients.push(client);
        } else {
            return Err(eyre::eyre!("Training not found"));
        }
        self.update_week(week).await
    }

    pub async fn sign_out_from_training(
        &self,
        training: &Training,
        client: ObjectId,
    ) -> Result<()> {
        let mut week = self.get_week(Some(training.start_at)).await?;
        let day = week.get_day_mut(training.start_at.weekday());

        let training = day.training.iter_mut().find(|slot| slot.id == training.id);

        if let Some(training) = training {
            training.clients.retain(|c| c != &client);
        } else {
            return Err(eyre::eyre!("Training not found"));
        }
        self.update_week(week).await
    }

    pub async fn get_day(&self, day: DateTime<Local>) -> Result<Day> {
        let week = self.calendar.get_week(day).await?;
        Ok(week.get_day(day.weekday()).clone())
    }

    pub async fn week_cursor(&self, date_time: DateTime<Local>) -> Result<mongodb::Cursor<Week>> {
        self.calendar.week_cursor(date_time).await
    }

    pub async fn update_week(&self, week: Week) -> Result<()> {
        self.calendar.update_week(week).await
    }

    pub async fn get_my_trainings(
        &self,
        client: ObjectId,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<Training>> {
        self.calendar.get_my_trainings(client, limit, offset).await
    }

    pub async fn is_free_time_slot(&self, start_at: DateTime<Local>) -> Result<bool, Error> {
        let week = self.calendar.get_week(start_at).await?;
        let day = week.get_day(start_at.weekday());
        for slot in &day.training {
            if slot.is_training_time(start_at) {
                return Ok(false);
            }
        }
        Ok(true)
    }
}
