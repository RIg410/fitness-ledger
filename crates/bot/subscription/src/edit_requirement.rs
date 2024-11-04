use async_trait::async_trait;
use bot_core::{
    callback_data::Calldata,
    calldata,
    context::Context,
    widget::{Jmp, View},
};
use eyre::Error;
use model::{rights::Rule, subscription::SubRequirements};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator as _;
use teloxide::types::InlineKeyboardMarkup;

pub struct EditRequirement {
    id: ObjectId,
}

impl EditRequirement {
    pub fn new(id: ObjectId) -> EditRequirement {
        EditRequirement { id }
    }
}

#[async_trait]
impl View for EditRequirement {
    fn name(&self) -> &'static str {
        "EditRequirement"
    }

    async fn show(&mut self, ctx: &mut Context) -> Result<(), Error> {
        ctx.ensure(Rule::EditSubscription)?;
        let mut keymap = InlineKeyboardMarkup::default();
        let msg = "*Выберите требование*";
        let sub = ctx
            .ledger
            .subscriptions
            .get(&mut ctx.session, self.id)
            .await?
            .ok_or_else(|| eyre::eyre!("Subscription not found"))?;

        if sub.requirements.is_none() {
            keymap = keymap.append_row(Callback::Select(None).btn_row("✅ нет"));
        } else {
            keymap = keymap.append_row(Callback::Select(None).btn_row("нет"));
        }

        let req_id = sub.requirements.map(|req| req.into_value()).unwrap_or(255);

        for req in SubRequirements::iter() {
            if req_id == req.into_value() {
                keymap = keymap.append_row(
                    Callback::Select(Some(req.into_value())).btn_row(format!("✅ {:?}", req)),
                );
            } else {
                keymap = keymap.append_row(
                    Callback::Select(Some(req.into_value())).btn_row(format!("{:?}", req)),
                );
            }
        }
        ctx.edit_origin(msg, keymap).await?;
        Ok(())
    }

    async fn handle_callback(&mut self, ctx: &mut Context, data: &str) -> Result<Jmp, Error> {
        ctx.ensure(Rule::EditSubscription)?;
        match calldata!(data) {
            Callback::Select(None) => {
                ctx.ledger
                    .subscriptions
                    .update_requirements(&mut ctx.session, self.id, None)
                    .await?;
            }
            Callback::Select(Some(req)) => {
                ctx.ledger
                    .subscriptions
                    .update_requirements(
                        &mut ctx.session,
                        self.id,
                        SubRequirements::from_value(req),
                    )
                    .await?;
            }
        }
        Ok(Jmp::Stay)
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
enum Callback {
    Select(Option<u8>),
}
