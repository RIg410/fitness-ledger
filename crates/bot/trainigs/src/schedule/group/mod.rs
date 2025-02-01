use bot_core::{context::Context, widget::Widget};
use bot_viewer::rooms::fmt_room;
use chrono::{DateTime, Local};
use eyre::Result;
use is_one_time::SetOneTime;
use model::{program::Program, rooms::Room};
use mongodb::bson::oid::ObjectId;
use set_date_time::SetDateTime;
use set_instructor::SetInstructor;
use set_room::SetRoom;
use teloxide::utils::markdown::escape;

mod finish;
mod is_one_time;
pub mod set_date_time;
mod set_instructor;
mod set_room;

#[derive(Default, Clone, Copy)]
pub struct ScheduleTrainingPreset {
    pub day: Option<DateTime<Local>>,
    pub date_time: Option<DateTime<Local>>,
    pub instructor: Option<ObjectId>,
    pub is_one_time: Option<bool>,
    pub room: Option<ObjectId>,
    pub program_id: Option<ObjectId>,
}

impl ScheduleTrainingPreset {
    pub fn with_day(day: DateTime<Local>) -> Self {
        ScheduleTrainingPreset {
            day: Some(day),
            date_time: None,
            instructor: None,
            is_one_time: None,
            room: None,
            program_id: None,
        }
    }

    pub fn program_id(&self) -> Result<ObjectId> {
        self.program_id
            .ok_or_else(|| eyre::eyre!("Program is not set"))
    }

    pub(crate) fn into_next_view(self) -> Widget {
        if self.instructor.is_none() {
            return SetInstructor::new(self).into();
        }
        if self.room.is_none() {
            return SetRoom::new(self).into();
        }
        if self.is_one_time.is_none() {
            return SetOneTime::new(self).into();
        }

        if self.date_time.is_none() {
            return SetDateTime::new(self).into();
        }

        finish::Finish::new(self).into()
    }
}

pub async fn render_msg(
    ctx: &mut Context,
    training: &Program,
    preset: &ScheduleTrainingPreset,
    request: &str,
) -> Result<String> {
    let date_time = if let Some(date_time) = preset.date_time {
        date_time.format("%d\\.%m %H:%M").to_string()
    } else if let Some(date) = preset.day {
        date.format("%d\\.%m ❓:❓").to_string()
    } else {
        "❓".to_string()
    };

    let instructor = if let Some(tg_id) = preset.instructor {
        let user = ctx
            .ledger
            .users
            .get(&mut ctx.session, tg_id)
            .await?
            .ok_or_else(|| eyre::eyre!("User not found"))?;
        if let Some(name) = &user.name.tg_user_name {
            format!("{}(@{})", user.name.first_name, name)
        } else {
            user.name.first_name.to_owned()
        }
    } else {
        "❓".to_string()
    };

    Ok(format!(
        "*Тренировка*: _{}_\n*Дата*: _{}_\n*Инструктор*: _{}_\n*Регулярность*: _{}_\n*Зал*: _{}_\n\n*{}*",
        escape(&training.name),
        date_time,
        escape(&instructor),
        preset
            .is_one_time
            .map(|b| if b {
                "разовая".to_owned()
            } else {
                "регулярная".to_owned()
            })
            .unwrap_or_else(|| "❓".to_string()),
        preset
            .room
            .map(|r| fmt_room(Room::from(r)))
            .unwrap_or_else(|| "❓"),
        escape(&if request.is_empty() {"."} else {request}),
    ))
}
