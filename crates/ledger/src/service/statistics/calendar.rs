use crate::service::{calendar::Calendar, users::Users};
use chrono::NaiveDate;
use eyre::Error;
use model::{
    rooms::Room,
    session::Session,
    statistics::{
        day::{DayStat, TrainingStat, TrainingType},
        month::MonthStatistics,
    },
};
use std::collections::HashMap;

use super::month_id;

pub async fn load_calendar(
    calendar: &Calendar,
    users: &Users,
    session: &mut Session,
) -> Result<HashMap<NaiveDate, MonthStatistics>, Error> {
    let mut days = calendar.find_range(session, None, None).await?;
    let mut monthes = HashMap::new();

    let mut instructors = HashMap::new();

    while let Some(day) = days.next(session).await {
        let day = day?;

        let month = month_id(day.day_date());
        let month = monthes
            .entry(month)
            .or_insert_with(|| MonthStatistics::default());

        let mut day_training = Vec::new();
        for training in &day.training {
            if !training.is_processed {
                continue;
            }

            let slot = training.get_slot();
            let room = match Room::from(training.get_slot().room()) {
                Room::Adult => "Большой зал",
                Room::Child => "Малый зал",
            };

            let tp = match training.tp {
                model::program::TrainingType::Group { .. } => TrainingType::Group,
                model::program::TrainingType::Personal { .. } => TrainingType::Personal,
                model::program::TrainingType::SubRent { .. } => TrainingType::Rent,
            };

            let instructor = instructors.get(&training.instructor).cloned();
            let instructor = if let Some(instructor) = instructor {
                instructor
            } else {
                let user = users
                    .get(session, training.instructor)
                    .await?
                    .map(|u| u.name.first_name);
                instructors.insert(training.instructor, user.clone());
                user
            };

            day_training.push(TrainingStat {
                name: training.name.clone(),
                start_at: slot.start_at(),
                clients: training.clients.len() as u32,
                instructor,
                earned: training
                    .statistics
                    .as_ref()
                    .map(|s| s.earned.int_part())
                    .unwrap_or_default(),
                paid: training
                    .statistics
                    .as_ref()
                    .map(|s| s.couch_rewards.int_part())
                    .unwrap_or_default(),
                tp,
                room: room.to_string(),
            });
        }

        day_training.sort_by(|a, b| a.start_at.cmp(&b.start_at));
        month.days.push(DayStat {
            dt: day.day_date(),
            trainings: day_training,
        });
    }

    Ok(monthes)
}
