use async_trait::async_trait;
use bot_core::{context::Context, widget::View};
use eyre::Result;
use model::decimal::Decimal;
use model::user::rate::Rate;
use mongodb::bson::oid::ObjectId;

pub struct PersonalRate {
    percent: Option<Decimal>,
    old_rate: Option<Rate>,
    user_id: ObjectId,
}

impl PersonalRate {
    pub fn new(old_rate: Option<Rate>, user_id: ObjectId) -> PersonalRate {
        PersonalRate {
            old_rate,
            percent: None,
            user_id,
        }
    }
}

#[async_trait]
impl View for PersonalRate {
    fn name(&self) -> &'static str {
        "PersonalRate"
    }

    async fn show(&mut self, ctx: &mut Context) -> Result<()> {
        todo!()
    }
}
