use async_trait::async_trait;
use bot_core::{
    callback_data::Calldata as _,
    calldata,
    context::Context,
    widget::{Jmp, View},
};
use bot_viewer::fmt_phone;
use model::rights::Rule;
use serde::{Deserialize, Serialize};
use teloxide::{types::InlineKeyboardMarkup, utils::markdown::escape};

#[derive(Default)]
pub struct ClientsStatistics;

impl ClientsStatistics {
    pub async fn send_clients_without_subscription(
        &self,
        ctx: &mut Context,
    ) -> Result<(), eyre::Error> {
        ctx.send_notification("Клиенты без абонемента").await;

        let users = ctx
            .ledger
            .statistics
            .find_clients_without_subscription(&mut ctx.session)
            .await?;

        let mut msg = String::new();
        let mut count = 0;
        for user in users {
            msg.push_str(&format!(
                "👤 *{}* {}\n",
                escape(&user.name.first_name),
                fmt_phone(user.phone.as_deref()),
            ));
            count += 1;

            if count > 20 {
                ctx.send_notification(&msg).await;
                msg.clear();
                count = 0;
            }
        }

        Ok(())
    }
}

#[async_trait]
impl View for ClientsStatistics {
    fn name(&self) -> &'static str {
        "ClientsStatistics"
    }

    async fn show(&mut self, ctx: &mut Context) -> Result<(), eyre::Error> {
        ctx.ensure(Rule::ViewStatistics)?;
        let msg = "Статистика по клиентам: 📊".to_string();
        let mut keymap = InlineKeyboardMarkup::default();
        keymap = keymap.append_row(vec![
            Callback::ClientsWithoutSubscription.button("Клиенты без абонемента")
        ]);

        ctx.edit_origin(&msg, keymap).await?;
        Ok(())
    }

    async fn handle_callback(&mut self, ctx: &mut Context, data: &str) -> Result<Jmp, eyre::Error> {
        ctx.ensure(Rule::ViewStatistics)?;

        match calldata!(data) {
            Callback::ClientsWithoutSubscription => {
                self.send_clients_without_subscription(ctx).await?;
            }
        }
        Ok(Jmp::Stay)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum Callback {
    ClientsWithoutSubscription,
}
