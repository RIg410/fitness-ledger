use async_trait::async_trait;
use bot_core::{context::Context, widget::View};
use eyre::Result;
use model::decimal::Decimal;
use model::user::rate::Rate;
use mongodb::bson::oid::ObjectId;

pub struct GroupRate {
    percent: Option<Decimal>,
    min_reward: Option<Decimal>,
    old_rate: Option<Rate>,
    user_id: ObjectId,
}

impl GroupRate {
    pub fn new(old_rate: Option<Rate>, user_id: ObjectId) -> GroupRate {
        GroupRate {
            old_rate,
            percent: None,
            min_reward: None,
            user_id,
        }
    }
}

#[async_trait]
impl View for GroupRate {
    fn name(&self) -> &'static str {
        "GroupRate"
    }

    async fn show(&mut self, ctx: &mut Context) -> Result<()> {
        todo!()
    }
}
