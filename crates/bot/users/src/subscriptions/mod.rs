use async_trait::async_trait;
use bot_core::{
    callback_data::Calldata as _,
    calldata,
    context::Context,
    widget::{Jmp, View},
};
use bot_viewer::user::render_sub;
use model::rights::Rule;
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use teloxide::types::InlineKeyboardMarkup;

pub struct SubscriptionsList {
    id: ObjectId,
    index: usize,
}

impl SubscriptionsList {
    pub fn new(id: ObjectId) -> Self {
        SubscriptionsList { id, index: 0 }
    }
}

#[async_trait]
impl View for SubscriptionsList {
    fn name(&self) -> &'static str {
        "SubscriptionsList"
    }

    async fn show(&mut self, ctx: &mut Context) -> Result<(), eyre::Error> {
        ctx.ensure(Rule::EditUserSubscription)?;

        let user = ctx.ledger.get_user(&mut ctx.session, self.id).await?;
        let payer = user.payer()?;
        let subs = payer.subscriptions();
        let mut txt = String::new();
        let mut keymap = InlineKeyboardMarkup::default();

        if subs.is_empty() {
            txt.push_str("_Нет абонементов_");
            return Ok(());
        } else {
            txt.push_str("Выберите абонемент:\n");
            for (i, sub) in subs.iter().enumerate() {
                let select = if i == self.index { "✅" } else { " " };
                txt.push_str(&format!(
                    "{} *{}*\n",
                    select,
                    render_sub(sub, payer.is_owner())
                ));
            }
        }

        if !subs.is_empty() {
            keymap = keymap.append_row(vec![
                Calldata::Select(self.index.saturating_sub(1)).button("⬆️"),
                Calldata::Select(self.index + 1).button("⬇️"),
            ]);
            keymap = keymap.append_row(vec![
                Calldata::ChangeBalance(-1).button("Уменьшить баланс"),
                Calldata::ChangeBalance(1).button("Увеличить баланс"),
            ]);
            keymap = keymap.append_row(vec![
                Calldata::ChangeLockBalance(-1).button("Уменьшить резерв"),
                Calldata::ChangeLockBalance(1).button("Увеличить резерв"),
            ]);
            keymap = keymap.append_row(vec![
                Calldata::ChangeDays(-1).button("Уменьшить дни"),
                Calldata::ChangeDays(1).button("Увеличить дни"),
            ]);
        }

        ctx.edit_origin(&txt, keymap).await?;
        Ok(())
    }

    async fn handle_callback(&mut self, ctx: &mut Context, data: &str) -> Result<Jmp, eyre::Error> {
        ctx.ensure(Rule::EditUserSubscription)?;
        let user = ctx.ledger.get_user(&mut ctx.session, self.id).await?;
        let payer = user.payer()?;

        match calldata!(data) {
            Calldata::Select(index) => {
                if index > payer.subscriptions().len() {
                    return Ok(Jmp::Stay);
                }
                self.index = index;
            }
            Calldata::ChangeBalance(delta) => {
                if self.index >= payer.subscriptions().len() {
                    return Ok(Jmp::Stay);
                }
                let sub = &payer.subscriptions()[self.index];
                ctx.ledger
                    .users
                    .change_subscription_balance(&mut ctx.session, self.id, sub.id, delta)
                    .await?;
            }
            Calldata::ChangeLockBalance(delta) => {
                if self.index >= payer.subscriptions().len() {
                    return Ok(Jmp::Stay);
                }
                let sub = &payer.subscriptions()[self.index];
                ctx.ledger
                    .users
                    .change_subscription_locked_balance(&mut ctx.session, self.id, sub.id, delta)
                    .await?;
            }
            Calldata::ChangeDays(delta) => {
                if self.index >= payer.subscriptions().len() {
                    return Ok(Jmp::Stay);
                }
                let sub = &payer.subscriptions()[self.index];
                ctx.ledger
                    .users
                    .change_subscription_days(&mut ctx.session, self.id, sub.id, delta)
                    .await?;
            }
        }

        Ok(Jmp::Stay)
    }
}

#[derive(Serialize, Deserialize)]
enum Calldata {
    Select(usize),
    ChangeBalance(i64),
    ChangeLockBalance(i64),
    ChangeDays(i64),
}
