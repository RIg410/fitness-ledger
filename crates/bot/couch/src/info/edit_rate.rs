use async_trait::async_trait;
use bot_core::{
    context::Context,
    script::{
        list::{ListId, ListItem, StageList},
        text::StageText,
        yes_no::StageYesNo,
        Dispatch, Stage,
    },
};
use bot_viewer::user::fmt_rate;
use chrono::{Local, NaiveDate, TimeZone as _, Utc};
use eyre::Error;
use model::{couch::Rate, decimal::Decimal};
use mongodb::bson::oid::ObjectId;

#[derive(Default)]
pub struct ChangeRateState {
    pub user: ObjectId,
    pub reward_rate: Option<Rate>,
}

pub struct SetRewardType;

#[async_trait]
impl StageList<ChangeRateState> for SetRewardType {
    async fn message(
        &self,
        _: &mut Context,
        _: &mut ChangeRateState,
        _: usize,
        _: usize,
    ) -> Result<(String, Vec<Vec<ListItem>>), Error> {
        let mut rewards = vec![];

        rewards.push(vec![ListItem {
            id: ListId::I64(0),
            name: "Без вознаграждения 🚫".to_string(),
        }]);
        rewards.push(vec![ListItem {
            id: ListId::I64(1),
            name: "Фиксированное вознаграждение 💵".to_string(),
        }]);
        rewards.push(vec![ListItem {
            id: ListId::I64(2),
            name: "По клиентам 👥".to_string(),
        }]);

        Ok(("Выберите тип вознаграждения 💰".to_string(), rewards))
    }

    async fn select(
        &self,
        _: &mut Context,
        state: &mut ChangeRateState,
        id: ListId,
    ) -> Result<Dispatch<ChangeRateState>, Error> {
        let id = id.as_i64().ok_or_else(|| eyre::eyre!("Invalid id"))?;
        Ok(match id {
            0 => {
                state.reward_rate = Some(Rate::None);
                Dispatch::Stage(Stage::yes_no(Confirm))
            }
            1 => Dispatch::Stage(Stage::text(FixedRate)),
            2 => Dispatch::Stage(Stage::text(ClientRate)),
            _ => return Ok(Dispatch::None),
        })
    }

    fn back(&self) -> Option<Stage<ChangeRateState>> {
        None
    }
}

pub struct FixedRate;

#[async_trait]
impl StageText<ChangeRateState> for FixedRate {
    async fn message(&self, _: &mut Context, _: &mut ChangeRateState) -> Result<String, eyre::Error> {
        Ok("Введите фиксированное вознаграждение 💵".to_string())
    }

    async fn handle_text(
        &self,
        _: &mut Context,
        state: &mut ChangeRateState,
        query: &str,
    ) -> Result<Dispatch<ChangeRateState>, Error> {
        let rate = query.parse::<Decimal>()?;
        let rate = Rate::FixedMonthly {
            rate,
            next_reward: Utc::now(),
        };
        state.reward_rate = Some(rate);
        Ok(Dispatch::Stage(Stage::text(FixedRateNextReward)))
    }

    fn back(&self) -> Option<Stage<ChangeRateState>> {
        Some(Stage::list(SetRewardType))
    }
}

struct ClientRate;

#[async_trait]
impl StageText<ChangeRateState> for ClientRate {
    async fn message(&self, _: &mut Context, _: &mut ChangeRateState) -> Result<String, eyre::Error> {
        Ok("Введите минимальное вознаграждение 💵".to_string())
    }

    async fn handle_text(
        &self,
        _: &mut Context,
        state: &mut ChangeRateState,
        query: &str,
    ) -> Result<Dispatch<ChangeRateState>, Error> {
        let min = query.parse::<Decimal>()?;

        state.reward_rate = Some(Rate::PerClient {
            min,
            per_client: Decimal::zero(),
        });
        Ok(Dispatch::Stage(Stage::text(ClientRatePerClient)))
    }

    fn back(&self) -> Option<Stage<ChangeRateState>> {
        Some(Stage::list(SetRewardType))
    }
}

struct ClientRatePerClient;

#[async_trait]
impl StageText<ChangeRateState> for ClientRatePerClient {
    async fn message(&self, _: &mut Context, _: &mut ChangeRateState) -> Result<String, eyre::Error> {
        Ok("Введите вознаграждение за клиента 💵".to_string())
    }

    async fn handle_text(
        &self,
        _: &mut Context,
        state: &mut ChangeRateState,
        query: &str,
    ) -> Result<Dispatch<ChangeRateState>, Error> {
        let per_client = query.parse::<Decimal>()?;

        if let Some(Rate::PerClient { min, .. }) = state.reward_rate.as_mut() {
            state.reward_rate = Some(Rate::PerClient {
                min: *min,
                per_client,
            });
        } else {
            eyre::bail!("Rate not found");
        }
        Ok(Dispatch::Stage(Stage::yes_no(Confirm)))
    }

    fn back(&self) -> Option<Stage<ChangeRateState>> {
        Some(Stage::text(ClientRate))
    }
}

struct FixedRateNextReward;

#[async_trait]
impl StageText<ChangeRateState> for FixedRateNextReward {
    async fn message(&self, _: &mut Context, _: &mut ChangeRateState) -> Result<String, eyre::Error> {
        Ok("Введите дату следующего вознаграждения 📅\nY\\.m\\.d".to_string())
    }

    async fn handle_text(
        &self,
        _: &mut Context,
        state: &mut ChangeRateState,
        query: &str,
    ) -> Result<Dispatch<ChangeRateState>, Error> {
        let next_reward = NaiveDate::parse_from_str(query, "%Y.%m.%d")?;
        let next_reward = Local
            .from_local_datetime(
                &next_reward
                    .and_hms_opt(0, 0, 0)
                    .ok_or_else(|| eyre::eyre!("Invalid time"))?,
            )
            .earliest()
            .ok_or_else(|| eyre::eyre!("Invalid time"))?;

        let rate = if let Some(Rate::FixedMonthly { rate, .. }) = state.reward_rate.as_mut() {
            Rate::FixedMonthly {
                rate: *rate,
                next_reward: next_reward.with_timezone(&Utc),
            }
        } else {
            eyre::bail!("Rate not found");
        };
        state.reward_rate = Some(rate);
        Ok(Dispatch::Stage(Stage::yes_no(Confirm)))
    }

    fn back(&self) -> Option<Stage<ChangeRateState>> {
        Some(Stage::list(SetRewardType))
    }
}

pub struct Confirm;

#[async_trait]
impl StageYesNo<ChangeRateState> for Confirm {
    async fn message(&self, _: &mut Context, state: &mut ChangeRateState) -> Result<String, Error> {
        let rate = if let Some(rate) = state.reward_rate.as_ref() {
            rate
        } else {
            eyre::bail!("User, description or rate not found");
        };
        Ok(format!("Вознаграждение: {}\n", fmt_rate(rate)))
    }

    async fn yes(&self, ctx: &mut Context, state: &mut ChangeRateState) -> Result<Dispatch<ChangeRateState>, Error> {
        let rate = if let Some(rate) = state.reward_rate.as_ref() {
            rate
        } else {
            eyre::bail!("User, description or rate not found");
        };
        ctx.ledger
            .users
            .update_couch_rate(&mut ctx.session, state.user, rate.clone())
            .await?;
        ctx.send_notification("Оплата изменена 🎉").await?;
        Ok(Dispatch::WidgetBack)
    }

    fn back(&self) -> Option<Stage<ChangeRateState>> {
        Some(Stage::list(SetRewardType))
    }
}
