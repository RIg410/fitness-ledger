use async_trait::async_trait;
use bot_core::{
    context::Context,
    script::{text::StageText, yes_no::StageYesNo, Dispatch, Stage},
};
use eyre::Error;
use model::{couch::PersonalRate, decimal::Decimal};
use mongodb::bson::oid::ObjectId;

pub struct ChangePersonalRateState {
    pub user: ObjectId,
    pub personal_rate: PersonalRate,
}

pub struct EditPersonalInterest;

#[async_trait]
impl StageText<ChangePersonalRateState> for EditPersonalInterest {
    async fn message(
        &self,
        _: &mut Context,
        _: &mut ChangePersonalRateState,
    ) -> Result<String, eyre::Error> {
        Ok("Введите процент вознаграждения от пересональной тренировки 💵".to_string())
    }

    async fn handle_text(
        &self,
        _: &mut Context,
        state: &mut ChangePersonalRateState,
        query: &str,
    ) -> Result<Dispatch<ChangePersonalRateState>, Error> {
        let couch_interest = query.parse::<Decimal>()?;
        state.personal_rate = PersonalRate { couch_interest };
        Ok(Dispatch::Stage(Stage::yes_no(Confirm)))
    }

    fn back(&self) -> Option<Stage<ChangePersonalRateState>> {
        None
    }
}

struct Confirm;

#[async_trait]
impl StageYesNo<ChangePersonalRateState> for Confirm {
    async fn message(&self, _: &mut Context, state: &mut ChangePersonalRateState) -> Result<String, Error> {
        Ok(format!(
            "Вы уверены, что хотите установить процент вознаграждения от персональной тренировки на {}%?",
            state.personal_rate.couch_interest
        ))
    }

    async fn yes(
        &self,
        ctx: &mut Context,
        state: &mut ChangePersonalRateState,
    ) -> Result<Dispatch<ChangePersonalRateState>, Error> {
        ctx.ledger
            .users
            .update_couch_personal_rate(
                &mut ctx.session,
                state.user.clone(),
                state.personal_rate.clone(),
            )
            .await?;
        ctx.send_notification("Процент вознаграждения от персональной тренировки успешно обновлен")
            .await?;
        Ok(Dispatch::WidgetBack)
    }

    fn back(&self) -> Option<Stage<ChangePersonalRateState>> {
        Some(Stage::text(EditPersonalInterest))
    }
}
