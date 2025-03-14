mod finish;
mod set_client;
mod set_date_time;
mod set_instructor;
mod set_room;

use bot_core::{context::Context, widget::Widget};
use bot_viewer::rooms::fmt_room;
use chrono::{DateTime, Local};
use eyre::Result;
use model::rooms::Room;
use mongodb::bson::oid::ObjectId;
use set_date_time::SetDateTime;
use set_instructor::SetInstructor;
use set_room::SetRoom;
use teloxide::utils::markdown::escape;

pub const DURATION: u32 = 60;

#[derive(Default, Clone, Copy)]
pub struct PersonalTrainingPreset {
    pub day: Option<DateTime<Local>>,
    pub date_time: Option<DateTime<Local>>,
    pub instructor: Option<ObjectId>,
    pub client: Option<ObjectId>,
    pub room: Option<ObjectId>,
}

impl PersonalTrainingPreset {
    pub fn with_day(day: DateTime<Local>) -> Self {
        PersonalTrainingPreset {
            day: Some(day),
            date_time: None,
            instructor: None,
            client: None,
            room: None,
        }
    }

    pub fn with_day_and_instructor(day: DateTime<Local>, instructor: ObjectId) -> Self {
        PersonalTrainingPreset {
            day: Some(day),
            date_time: None,
            instructor: Some(instructor),
            room: None,
            client: None,
        }
    }

    pub fn into_next_view(self) -> Widget {
        if self.instructor.is_none() {
            return SetInstructor::new(self).into();
        }
        if self.room.is_none() {
            return SetRoom::new(self).into();
        }
        if self.date_time.is_none() {
            return SetDateTime::new(self).into();
        }
        if self.client.is_none() {
            return set_client::SetClient::new(self).into();
        }

        finish::Finish::new(self).into()
    }
}

pub async fn render_msg(
    ctx: &mut Context,
    preset: &PersonalTrainingPreset,
    request: &str,
) -> Result<String> {
    let date_time = if let Some(date_time) = preset.date_time {
        date_time.format("%d\\.%m %H:%M").to_string()
    } else if let Some(date) = preset.day {
        date.format("%d\\.%m ❓:❓").to_string()
    } else {
        "❓".to_string()
    };

    let instructor = if let Some(id) = preset.instructor {
        let user = ctx
            .ledger
            .users
            .get(&mut ctx.session, id)
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

    let client = if let Some(id) = preset.client {
        let user = ctx
            .ledger
            .users
            .get(&mut ctx.session, id)
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
        "*Персональная тренировка*\n*Дата*: _{}_\n*Инструктор*: _{}_\n*Клиент*: _{}_\n*Зал*: _{}_\n\n*{}*",
        date_time,
        escape(&instructor),
        escape(&client),
        preset
            .room
            .map(|r| fmt_room(Room::from(r)))
            .unwrap_or_else(|| "❓"),
        escape(if request.is_empty() { "." } else { request }),
    ))
}
