use super::View;
use crate::{context::Context, state::Widget};
use chrono::{DateTime, Local};
use eyre::Result;
use is_one_time::SetOneTime;
use model::proto::TrainingProto;
use mongodb::bson::oid::ObjectId;
use set_date_time::SetDateTime;
use set_instructor::SetInstructor;
use teloxide::utils::markdown::escape;

mod finish;
mod is_one_time;
mod set_date_time;
mod set_instructor;

#[derive(Default, Clone)]
pub struct ScheduleTrainingPreset {
    pub day: Option<DateTime<Local>>,
    pub date_time: Option<DateTime<Local>>,
    pub instructor: Option<i64>,
    pub is_one_time: Option<bool>,
}

impl ScheduleTrainingPreset {
    pub fn into_next_view(self, id: ObjectId, go_back: Widget) -> Widget {
        if self.date_time.is_none() {
            return Box::new(SetDateTime::new(id, self, go_back));
        }

        if self.instructor.is_none() {
            return Box::new(SetInstructor::new(id, self, go_back));
        }

        if self.is_one_time.is_none() {
            return Box::new(SetOneTime::new(id, self, go_back));
        }

        Box::new(finish::Finish::new(id, self, go_back))
    }
}

pub async fn render_msg(
    ctx: &mut Context,
    training: &TrainingProto,
    preset: &ScheduleTrainingPreset,
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
            .get_by_tg_id(&mut ctx.session, tg_id)
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
        "*Тренировка*: _{}_\n*Дата*: _{}_\n*Инструктор*: _{}_\n*Регулярность*: _{}_",
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
    ))
}
