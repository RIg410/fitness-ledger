use async_trait::async_trait;
use bot_core::{
    callback_data::Calldata as _,
    calldata,
    context::Context,
    widget::{Jmp, View},
};
use bot_viewer::user::render_sub;
use model::rights::Rule;
use serde::{Deserialize, Serialize};
use teloxide::types::InlineKeyboardMarkup;

pub struct SubscriptionsList {
    id: i64,
    index: usize,
}

impl SubscriptionsList {
    pub fn new(id: i64) -> Self {
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

        let subs = &user.subscriptions;
        let mut txt = String::new();
        let mut keymap = InlineKeyboardMarkup::default();

        if subs.is_empty() {
            txt.push_str("_Нет абонементов_");
            return Ok(());
        } else {
            txt.push_str("Выберите абонемент:\n");
            for (i, sub) in subs.iter().enumerate() {
                let select = if i == self.index { "✅" } else { " " };
                txt.push_str(&format!("{} *{}*\n", select, render_sub(sub)));
            }
        }

        if subs.len() >= 1 {
            keymap = keymap.append_row(vec![
                Calldata::Select(self.index.saturating_sub(1)).button("⬆️"),
                Calldata::Select(self.index + 1).button("⬇️"),
            ]);
            keymap = keymap.append_row(vec![
                Calldata::ChangeBalance(-1).button("Уменьшить баланс"),
                Calldata::ChangeBalance(1).button("Увеличить баланс"),
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

        match calldata!(data) {
            Calldata::Select(index) => {
                if index > user.subscriptions.len() {
                    return Ok(Jmp::Stay);
                }
                self.index = index;
            }
            Calldata::ChangeBalance(delta) => {
                let sub = &user.subscriptions[self.index];
                ctx.ledger
                    .users
                    .change_subscription_balance(&mut ctx.session, self.id, sub.id, delta)
                    .await?;
            }
            Calldata::ChangeDays(delta) => {
                let sub = &user.subscriptions[self.index];
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
    ChangeDays(i64),
}
