mod finish;
mod set_date_time;
mod set_duration;
mod set_price;
mod set_renter;
mod set_room;

use bot_core::{context::Context, widget::Widget};
use bot_viewer::rooms::fmt_room;
use chrono::{DateTime, Duration, Local};
use eyre::Result;
use model::{decimal::Decimal, rooms::Room};
use mongodb::bson::oid::ObjectId;
use set_date_time::SetDateTime;
use set_duration::SetDuration;
use set_room::SetRoom;

#[derive(Default, Clone)]
pub struct RentPreset {
    pub day: Option<DateTime<Local>>,
    pub date_time: Option<DateTime<Local>>,
    pub room: Option<ObjectId>,
    pub duration: Option<Duration>,
    pub price: Option<Decimal>,
    pub renter: Option<String>,
}

impl RentPreset {
    pub fn with_day(day: DateTime<Local>) -> Self {
        RentPreset {
            day: Some(day),
            date_time: None,
            room: None,
            duration: None,
            price: None,
            renter: None,
        }
    }

    pub fn into_next_view(self) -> Widget {
        if self.room.is_none() {
            return SetRoom::new(self).into();
        }
        if self.duration.is_none() {
            return SetDuration::new(self).into();
        }
        if self.date_time.is_none() {
            return SetDateTime::new(self).into();
        }
        if self.price.is_none() {
            return set_price::SetPrice::new(self).into();
        }
        if self.renter.is_none() {
            return set_renter::SetRenter::new(self).into();
        }

        finish::Finish::new(self).into()
    }
}

pub async fn render_msg(_: &mut Context, preset: &RentPreset, request: &str) -> Result<String> {
    let date_time = if let Some(date_time) = preset.date_time {
        date_time.format("%d\\.%m %H:%M").to_string()
    } else if let Some(date) = preset.day {
        date.format("%d\\.%m ❓:❓").to_string()
    } else {
        "❓".to_string()
    };

    let room = preset
        .room
        .map(|r| fmt_room(Room::from(r)))
        .unwrap_or_else(|| "❓");

    let duration = preset
        .duration
        .map(|d| format!("_{}_ мин", d.num_minutes()))
        .unwrap_or_else(|| "❓".to_string());

    let price = preset
        .price
        .map(|p| format!("_{}_", p))
        .unwrap_or_else(|| "❓".to_string());

    let renter = preset.renter.as_deref().unwrap_or("❓");

    Ok(format!(
        "*Субаренда*\n*Дата*: _{}_\n*Зал*: _{}_\n*Длительность*: {}\n*Цена*: {}\n*Арендатор*: _{}_\n\n{}",
        date_time,
        room,
        duration,
        price,
        renter,
        request
    ))
}
