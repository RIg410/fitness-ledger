use chrono::{DateTime, Local, Weekday};
use eyre::{Error, Result};
use model::{
    day::Day,
    ids::{DayId, WeekId},
    training::{Training, TrainingStatus},
};
use mongodb::{bson::oid::ObjectId, ClientSession};
use storage::calendar::CalendarStore;

#[derive(Clone)]
pub struct Calendar {
    calendar: CalendarStore,
}

impl Calendar {
    pub(crate) fn new(calendar: CalendarStore) -> Self {
        Calendar { calendar }
    }

    pub async fn get_day(&self, session: &mut ClientSession, day: DayId) -> Result<Day> {
        self.calendar.get_day(session, day).await
    }

    pub async fn get_week(&self, session: &mut ClientSession, id: WeekId) -> Result<Week> {
        let mon = id.day(chrono::Weekday::Mon);
        let tue = mon.next();
        let wed = tue.next();
        let thu = wed.next();
        let fri = thu.next();
        let sat = fri.next();
        let sun = sat.next();

        let week = [
            self.get_day(session, mon).await?,
            self.get_day(session, tue).await?,
            self.get_day(session, wed).await?,
            self.get_day(session, thu).await?,
            self.get_day(session, fri).await?,
            self.get_day(session, sat).await?,
            self.get_day(session, sun).await?,
        ];
        Ok(Week { id: id, days: week })
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

    pub async fn cancel_training(
        &self,
        session: &mut ClientSession,
        training: &Training,
    ) -> Result<()> {
        let mut day = self.get_day(session, training.day_id()).await?;
        let training = day.training.iter_mut().find(|slot| slot.id == training.id);

        if let Some(training) = training {
            training.status = TrainingStatus::Cancelled;
        } else {
            return Err(eyre::eyre!("Training not found"));
        }
        self.calendar.update_day(session, &day).await
    }

    pub async fn uncancel_training(
        &self,
        session: &mut ClientSession,
        training: &Training,
    ) -> Result<()> {
        let mut day = self.get_day(session, training.day_id()).await?;
        let training = day.training.iter_mut().find(|slot| slot.id == training.id);

        if let Some(training) = training {
            if training.status != TrainingStatus::Cancelled {
                return Err(eyre::eyre!("Training is not cancelled"));
            }
            training.status = TrainingStatus::OpenToSignup;
        } else {
            return Err(eyre::eyre!("Training not found"));
        }
        self.calendar.update_day(session, &day).await
    }

    pub async fn sign_up_for_training(
        &self,
        session: &mut ClientSession,
        training: &Training,
        client: ObjectId,
    ) -> Result<()> {
        let mut day = self.get_day(session, training.day_id()).await?;
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
        self.calendar.update_day(session, &day).await
    }

    pub async fn sign_out_from_training(
        &self,
        session: &mut ClientSession,
        training: &Training,
        client: ObjectId,
    ) -> Result<()> {
        let mut day = self.get_day(session, training.day_id()).await?;
        let training = day.training.iter_mut().find(|slot| slot.id == training.id);

        if let Some(training) = training {
            if training.status != TrainingStatus::OpenToSignup {
                return Err(eyre::eyre!("Training is not open to sign up"));
            }

            training.clients.retain(|c| c != &client);
        } else {
            return Err(eyre::eyre!("Training not found"));
        }
        self.calendar.update_day(session, &day).await
    }

    pub async fn get_my_trainings(
        &self,
        session: &mut ClientSession,
        client: ObjectId,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<Training>> {
        self.calendar
            .get_my_trainings(session, client, limit, offset)
            .await
    }

    pub async fn is_free_time_slot(
        &self,
        session: &mut ClientSession,
        start_at: DateTime<Local>,
    ) -> Result<bool, Error> {
        let id = DayId::new(start_at);
        let day = self.calendar.get_day(session, id).await?;
        for slot in &day.training {
            if slot.is_training_time(start_at) {
                return Ok(false);
            }
        }
        Ok(true)
    }

    pub async fn update_day(&self, session: &mut ClientSession, day: &Day) -> Result<()> {
        self.calendar.update_day(session, day).await
    }

    pub async fn cursor(
        &self,
        session: &mut ClientSession,
        from: DayId,
        week_day: Weekday,
    ) -> Result<mongodb::SessionCursor<Day>> {
        self.calendar.cursor(session, from, week_day).await
    }
}

pub struct Week {
    pub id: WeekId,
    pub days: [Day; 7],
}
