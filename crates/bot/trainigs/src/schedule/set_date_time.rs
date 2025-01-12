use super::{render_msg, ScheduleTrainingPreset};
use async_trait::async_trait;
use bot_core::{
    context::Context,
    widget::{Jmp, View},
};
use chrono::{DateTime, Datelike as _, Local, TimeZone, Timelike};
use eyre::{Error, Result};
use ledger::service::calendar::TimeSlotCollision;
use log::warn;
use mongodb::bson::oid::ObjectId;
use teloxide::{types::Message, utils::html::escape};

#[derive(Default)]
pub struct SetDateTime {
    id: ObjectId,
    preset: Option<ScheduleTrainingPreset>,
}

impl SetDateTime {
    pub fn new(id: ObjectId, preset: ScheduleTrainingPreset) -> Self {
        Self {
            id,
            preset: Some(preset),
        }
    }
}

#[async_trait]
impl View for SetDateTime {
    fn name(&self) -> &'static str {
        "SetDateTime"
    }

    async fn show(&mut self, ctx: &mut Context) -> Result<()> {
        let training = ctx
            .ledger
            .programs
            .get_by_id(&mut ctx.session, self.id)
            .await?
            .ok_or_else(|| eyre::eyre!("Training not found"))?;
        let msg = render_msg(ctx, &training, self.preset.as_ref().unwrap()).await?;
        ctx.send_msg(&msg).await?;

        if self.preset.as_ref().unwrap().day.is_none() {
            ctx.send_msg("На какой день назначить тренировку? _дд\\.мм_")
                .await?;
        } else {
            ctx.send_msg("На какое время назначить тренировку? _чч\\:мм_")
                .await?;
        }
        Ok(())
    }

    async fn handle_message(&mut self, ctx: &mut Context, message: &Message) -> Result<Jmp> {
        let msg = if let Some(msg) = message.text() {
            msg
        } else {
            return Ok(Jmp::Stay);
        };

        let parts = match TimeParts::try_from(msg) {
            Ok(parts) => parts,
            Err(err) => {
                warn!("Invalid time format: {}", err);
                ctx.send_msg("Неверный формат времени\\.").await?;
                return Ok(Jmp::Stay);
            }
        };

        if self.preset.as_ref().unwrap().day.is_none() {
            if let Ok(day) = parts.to_date() {
                let mut preset = self.preset.take().unwrap();
                preset.day = Some(day);
                return Ok(preset.into_next_view(self.id).into());
            } else {
                ctx.send_msg("Неверный формат даты\\. _дд\\.мм_").await?;
            }
        } else {
            let mut preset = self.preset.take().unwrap();
            let day = preset.day.unwrap();
            let date_time = day.with_hour(parts.0).and_then(|d| d.with_minute(parts.1));

            if let Some(date_time) = date_time {
                if let Some(collision) = ctx
                    .ledger
                    .calendar
                    .check_time_slot(
                        &mut ctx.session,
                        self.id,
                        date_time,
                        preset.room.unwrap(),
                        preset.is_one_time.unwrap_or(true),
                    )
                    .await?
                {
                    ctx.send_msg(&render_time_slot_collision(&collision))
                        .await?;
                    preset.date_time = None;
                } else {
                    preset.date_time = Some(date_time);
                }
                return Ok(preset.into_next_view(self.id).into());
            } else {
                ctx.send_msg("Неверный формат времени\\. _чч\\:мм_").await?;
            }
        }
        Ok(Jmp::Stay)
    }
}

struct TimeParts(u32, u32);

impl TryFrom<&str> for TimeParts {
    type Error = eyre::Error;

    fn try_from(value: &str) -> Result<Self> {
        let parts = if value.contains(":") {
            value.split(':').collect::<Vec<_>>()
        } else {
            value.split('.').collect::<Vec<_>>()
        };
        if parts.len() != 2 {
            return Err(eyre::eyre!("Invalid time format"));
        }
        let hour = parts[0].parse::<u32>()?;
        let minute = parts[1].parse::<u32>()?;
        Ok(Self(hour, minute))
    }
}

impl TimeParts {
    pub fn to_date(&self) -> Result<DateTime<Local>, Error> {
        let year = chrono::Local::now().naive_local().year_ce().1;
        Local
            .with_ymd_and_hms(
                year as i32,
                self.0.saturating_sub(1),
                self.1.saturating_sub(1),
                0,
                0,
                0,
            )
            .single()
            .ok_or_else(|| eyre::eyre!("Invalid time"))
    }
}

pub fn render_time_slot_collision(collision: &TimeSlotCollision) -> String {
    format!(
        "Это время уже занято другой тренировкой: {}\n\nДата:{}",
        escape(&collision.name),
        collision.get_slot().start_at().format("%d\\.%m %H:%M")
    )
}
