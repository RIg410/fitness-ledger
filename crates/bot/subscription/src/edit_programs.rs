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
    id: ObjectId,
}

impl EditPrograms {
    pub fn new(id: ObjectId) -> EditPrograms {
        EditPrograms { id }
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

        let subscription = ctx
            .ledger
            .subscriptions
            .get(&mut ctx.session, self.id)
            .await?
            .ok_or_else(|| eyre::eyre!("Subscription not found"))?;

        let programs = ctx.ledger.programs.get_all(&mut ctx.session, false).await?;

        if let SubscriptionType::Group { program_filter } = subscription.subscription_type {
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
                    .subscriptions
                    .edit_program_list(&mut ctx.session, self.id, program_id, true)
                    .await?;
            }
            Callback::Unselect(program_id) => {
                let program_id = ObjectId::from_bytes(program_id);
                ctx.ledger
                    .subscriptions
                    .edit_program_list(&mut ctx.session, self.id, program_id, false)
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
