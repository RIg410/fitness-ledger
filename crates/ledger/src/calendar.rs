use chrono::{DateTime, Local, Weekday};
use eyre::{Error, Result};
use model::{day::Day, ids::{DayId, WeekId}, training::{Training, TrainingStatus}};
use mongodb::bson::oid::ObjectId;
use storage::calendar::CalendarStore;

#[derive(Clone)]
pub struct Calendar {
    calendar: CalendarStore,
}

impl Calendar {
    pub(crate) fn new(calendar: CalendarStore) -> Self {
        Calendar { calendar }
    }

    pub async fn get_day(&self, day: DayId) -> Result<Day> {
        self.calendar.get_day(day).await
    }

    pub async fn get_week(&self, id: WeekId) -> Result<Week> {
        let mon = id.day(chrono::Weekday::Mon);
        let tue = mon.next();
        let wed = tue.next();
        let thu = wed.next();
        let fri = thu.next();
        let sat = fri.next();
        let sun = sat.next();
        let (mon, tue, wed, thu, fri, sat, sun) = tokio::try_join!(
            self.get_day(mon),
            self.get_day(tue),
            self.get_day(wed),
            self.get_day(thu),
            self.get_day(fri),
            self.get_day(sat),
            self.get_day(sun),
        )?;

        let week = [mon, tue, wed, thu, fri, sat, sun];
        Ok(Week { id: id, days: week })
    }

    pub async fn get_training_by_start_at(
        &self,
        id: DateTime<Local>,
    ) -> Result<Option<Training>, Error> {
        let day = self.get_day(DayId::from(id)).await?;
        Ok(day
            .training
            .iter()
            .find(|slot| slot.start_at == id)
            .map(|slot| slot.clone()))
    }

    pub async fn cancel_training(&self, training: &Training) -> Result<()> {
        let mut day = self.get_day(training.day_id()).await?;
        let training = day.training.iter_mut().find(|slot| slot.id == training.id);

        if let Some(training) = training {
            training.status = TrainingStatus::Cancelled;
        } else {
            return Err(eyre::eyre!("Training not found"));
        }
        self.calendar.update_day(&day).await
    }

    pub async fn uncancel_training(&self, training: &Training) -> Result<()> {
        let mut day = self.get_day(training.day_id()).await?;
        let training = day.training.iter_mut().find(|slot| slot.id == training.id);

        if let Some(training) = training {
            if training.status != TrainingStatus::Cancelled {
                return Err(eyre::eyre!("Training is not cancelled"));
            }
            training.status = TrainingStatus::OpenToSignup;
        } else {
            return Err(eyre::eyre!("Training not found"));
        }
        self.calendar.update_day(&day).await
    }

    pub async fn sign_up_for_training(&self, training: &Training, client: ObjectId) -> Result<()> {
        let mut day = self.get_day(training.day_id()).await?;
        let training = day.training.iter_mut().find(|slot| slot.id == training.id);

        if let Some(training) = training {
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
        self.calendar.update_day(&day).await
    }

    pub async fn sign_out_from_training(
        &self,
        training: &Training,
        client: ObjectId,
    ) -> Result<()> {
        let mut day = self.get_day(training.day_id()).await?;
        let training = day.training.iter_mut().find(|slot| slot.id == training.id);

        if let Some(training) = training {
            if training.status != TrainingStatus::OpenToSignup {
                return Err(eyre::eyre!("Training is not open to sign up"));
            }

            training.clients.retain(|c| c != &client);
        } else {
            return Err(eyre::eyre!("Training not found"));
        }
        self.calendar.update_day(&day).await
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
        let id = DayId::new(start_at);
        let day = self.calendar.get_day(id).await?;
        for slot in &day.training {
            if slot.is_training_time(start_at) {
                return Ok(false);
            }
        }
        Ok(true)
    }

    pub async fn update_day(&self, day: &Day) -> Result<()> {
        self.calendar.update_day(day).await
    }

    pub async fn cursor(&self, from: DayId, week_day: Weekday) -> Result<mongodb::Cursor<Day>> {
        self.calendar.cursor(from, week_day).await
    }
}

pub struct Week {
    pub id: WeekId,
    pub days: [Day; 7],
}
