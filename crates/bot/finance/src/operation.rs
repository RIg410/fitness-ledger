use async_trait::async_trait;
use bot_core::{
    callback_data::Calldata as _,
    calldata,
    context::Context,
    widget::{Jmp, View},
};
use bot_viewer::{day::fmt_dt, user::fmt_come_from};
use chrono::Local;
use eyre::eyre;
use model::{rights::Rule, treasury::TreasuryEvent};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use teloxide::{types::InlineKeyboardMarkup, utils::markdown::escape};

pub struct FinanceOperation {
    id: ObjectId,
}

impl FinanceOperation {
    pub fn new(id: ObjectId) -> FinanceOperation {
        FinanceOperation { id }
    }
}

#[async_trait]
impl View for FinanceOperation {
    fn name(&self) -> &'static str {
        "FinanceOperation"
    }

    async fn show(&mut self, ctx: &mut Context) -> Result<(), eyre::Error> {
        ctx.ensure(Rule::MakePayment)?;

        let event = ctx
            .ledger
            .treasury
            .get(&mut ctx.session, self.id)
            .await?
            .ok_or_else(|| eyre!("No treasury"))?;
        let msg = render_event(ctx, &event).await?;

        let mut keymap = InlineKeyboardMarkup::default();
        if ctx.has_right(Rule::DeleteHistory) {
            keymap = keymap.append_row(vec![Callback::Delete.button("🗑️ Удалить")]);
        }
        ctx.edit_origin(&msg, keymap).await?;
        Ok(())
    }

    async fn handle_callback(&mut self, ctx: &mut Context, data: &str) -> Result<Jmp, eyre::Error> {
        ctx.ensure(Rule::MakePayment)?;

        match calldata!(data) {
            Callback::Delete => {
                ctx.ensure(Rule::DeleteHistory)?;
                ctx.ledger
                    .treasury
                    .remove(&mut ctx.session, self.id)
                    .await?;
                Ok(Jmp::Back)
            }
        }
    }
}

async fn render_event(ctx: &mut Context, event: &TreasuryEvent) -> Result<String, eyre::Error> {
    let env_text = match &event.event {
        model::treasury::Event::SellSubscription(sell_subscription) => {
            let user = match sell_subscription.buyer_id.clone() {
                model::treasury::subs::UserId::Id(object_id) => ctx
                    .ledger
                    .get_user(&mut ctx.session, object_id)
                    .await
                    .ok()
                    .map(|user| user.name.to_string())
                    .unwrap_or_else(|| "-".to_string()),
                model::treasury::subs::UserId::Phone(phone) => phone.to_owned(),
                model::treasury::subs::UserId::None => "-".to_string(),
            };

            format!(
                "🛒 Продажа абонемента: {}р пользователю {}",
                event.sum(),
                user
            )
        }
        model::treasury::Event::Reward(user_id) => {
            let user = match user_id {
                model::treasury::subs::UserId::Id(object_id) => ctx
                    .ledger
                    .get_user(&mut ctx.session, *object_id)
                    .await
                    .ok()
                    .map(|user| user.name.to_string())
                    .unwrap_or_else(|| "-".to_string()),
                model::treasury::subs::UserId::Phone(phone) => phone.to_owned(),
                model::treasury::subs::UserId::None => "-".to_string(),
            };
            format!("🎁 Выплата награды: {} пользователю {}", event.sum(), user)
        }
        model::treasury::Event::Outcome(outcome) => {
            format!(
                "📉 Расход: {} руб.\nОписание: {}",
                event.sum(),
                outcome.description
            )
        }
        model::treasury::Event::Income(income) => {
            format!(
                "📈 Поступление: {} руб.\nОписание:{}",
                event.sum(),
                income.description
            )
        }
        model::treasury::Event::SubRent { .. } => {
            format!("🏠 Субаренда: {} руб.", event.sum())
        }
        model::treasury::Event::Rent => {
            format!("🏠 Аренда: {} руб.", event.sum())
        }
        model::treasury::Event::Marketing(come_from) => {
            format!(
                "📊 Маркетинг: {} руб. ({})",
                event.sum(),
                fmt_come_from(*come_from)
            )
        }
    };

    Ok(format!(
        "📅 {}\n{}",
        fmt_dt(&event.date_time.with_timezone(&Local)),
        escape(&env_text)
    ))
}

#[derive(Serialize, Deserialize)]
enum Callback {
    Delete,
}
