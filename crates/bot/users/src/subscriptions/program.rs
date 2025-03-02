use async_trait::async_trait;
use bot_core::{
    callback_data::Calldata as _,
    calldata,
    context::Context,
    widget::{Jmp, View},
};
use eyre::{bail, Error};
use model::{rights::Rule, subscription::SubscriptionType};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use teloxide::{types::InlineKeyboardMarkup, utils::markdown::escape};

pub struct EditPrograms {
    user_id: ObjectId,
    id: ObjectId,
}

impl EditPrograms {
    pub fn new(user_id: ObjectId, id: ObjectId) -> EditPrograms {
        EditPrograms { user_id, id }
    }
}

#[async_trait]
impl View for EditPrograms {
    fn name(&self) -> &'static str {
        "EditRequirement"
    }

    async fn show(&mut self, ctx: &mut Context) -> Result<(), Error> {
        ctx.ensure(Rule::EditSubscription)?;
        let mut keymap = InlineKeyboardMarkup::default();
        let msg = "*Выберите программы*";

        let user = ctx.ledger.get_user(&mut ctx.session, self.user_id).await?;
        let payer = user.payer()?;
        let subscription = payer
            .subscriptions()
            .iter()
            .find(|s| s.id == self.id)
            .ok_or_else(|| eyre::eyre!("Subscription not found"))?;

        let programs = ctx.ledger.programs.get_all(&mut ctx.session, false).await?;

        if let SubscriptionType::Group { program_filter } = &subscription.tp {
            for program in programs {
                let selected = program_filter.contains(&program.id);
                let callback = if selected {
                    Callback::Unselect(program.id.bytes())
                } else {
                    Callback::Select(program.id.bytes())
                };
                keymap = keymap.append_row(vec![callback.button(format!(
                    "{} {}",
                    if selected { "✅" } else { "❌" },
                    escape(&program.name)
                ))]);
            }
        } else {
            bail!("Only group subscriptions can have programs");
        }

        ctx.edit_origin(msg, keymap).await?;
        Ok(())
    }

    async fn handle_callback(&mut self, ctx: &mut Context, data: &str) -> Result<Jmp, Error> {
        ctx.ensure(Rule::EditSubscription)?;
        match calldata!(data) {
            Callback::Select(program_id) => {
                let program_id = ObjectId::from_bytes(program_id);
                ctx.ledger
                    .users
                    .change_subscription_program(
                        &mut ctx.session,
                        self.user_id,
                        self.id,
                        program_id,
                        true,
                    )
                    .await?;
            }
            Callback::Unselect(program_id) => {
                let program_id = ObjectId::from_bytes(program_id);
                ctx.ledger
                    .users
                    .change_subscription_program(
                        &mut ctx.session,
                        self.user_id,
                        self.id,
                        program_id,
                        false,
                    )
                    .await?;
            }
        }
        Ok(Jmp::Stay)
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
enum Callback {
    Select([u8; 12]),
    Unselect([u8; 12]),
}
