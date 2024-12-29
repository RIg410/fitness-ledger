use async_trait::async_trait;
use bot_core::{context::Context, widget::View};
use chrono::{DateTime, Duration, Local};
use eyre::Result;
use model::{decimal::Decimal, user::rate::Rate};
use mongodb::bson::oid::ObjectId;

pub struct FixRate {
    amount: Option<Decimal>,
    next_payment_date: Option<DateTime<Local>>,
    interval: Option<Duration>,
    old_rate: Option<Rate>,
    user_id: ObjectId,
}

impl FixRate {
    pub fn new(old_rate: Option<Rate>, user_id: ObjectId) -> FixRate {
        FixRate {
            old_rate,
            amount: None,
            next_payment_date: None,
            interval: None,
            user_id,
        }
    }
}

#[async_trait]
impl View for FixRate {
    fn name(&self) -> &'static str {
        "FixRate"
    }

    async fn show(&mut self, ctx: &mut Context) -> Result<()> {
        todo!()
    }
}
