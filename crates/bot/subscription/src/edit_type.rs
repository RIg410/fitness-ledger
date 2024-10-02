use async_trait::async_trait;
use bot_core::{
    callback_data::Calldata,
    calldata,
    context::Context,
    widget::{Jmp, View},
};
use bot_viewer::subscription::fmt_subscription_type;
use eyre::Error;
use model::{rights::Rule, subscription::SubscriptionType};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use teloxide::types::InlineKeyboardMarkup;

pub struct EditSubscriptionType {
    id: ObjectId,
    tp: Option<SubscriptionType>,
    confirmed: bool,
}

impl EditSubscriptionType {
    pub fn new(id: ObjectId) -> Self {
        Self {
            id,
            tp: None,
            confirmed: false,
        }
    }
}

#[async_trait]
impl View for EditSubscriptionType {
    fn name(&self) -> &'static str {
        "EditSubscriptionType"
    }

    async fn show(&mut self, ctx: &mut Context) -> Result<(), Error> {
        ctx.ensure(Rule::EditSubscription)?;
        let mut keymap = InlineKeyboardMarkup::default();

        if self.confirmed {
            let msg = format!(
                "{}\nПодтвердите изменения:",
                fmt_subscription_type(ctx, self.tp.as_ref().unwrap()).await?
            );
            keymap = keymap.append_row(vec![
                Callback::Confirm(true).button("✅ Подтвердить"),
                Callback::Confirm(false).button("❌ Отменить"),
            ]);
            ctx.edit_origin(&msg, keymap).await?;
            return Ok(());
        }

        match self.tp {
            Some(_) => {
                let msg: String = "*Выберите инструктора*".to_string();
                let couch_list = ctx.ledger.users.instructors(&mut ctx.session).await?;
                for couch in couch_list {
                    keymap =
                        keymap
                            .append_row(vec![Callback::Couch(Some(couch.id.bytes()))
                                .button(&couch.name.first_name)]);
                }
                keymap = keymap.append_row(vec![Callback::Couch(None).button("Без инструктора")]);
                ctx.edit_origin(&msg, keymap).await?;
            }
            None => {
                let msg = "Выберите тип абонемента:".to_string();
                keymap = keymap.append_row(vec![
                    Callback::Group(true).button("Груповой"),
                    Callback::Group(false).button("Индивидуальный"),
                ]);
                ctx.edit_origin(&msg, keymap).await?;
            }
        }
        Ok(())
    }

    async fn handle_callback(&mut self, ctx: &mut Context, data: &str) -> Result<Jmp, Error> {
        ctx.ensure(Rule::EditSubscription)?;
        match calldata!(data) {
            Callback::Group(group) => {
                if group {
                    self.tp = Some(SubscriptionType::Group {});
                    self.confirmed = true;
                } else {
                    self.tp = Some(SubscriptionType::Personal { couch_filter: None });
                }
            }
            Callback::Couch(couch) => {
                self.tp = Some(SubscriptionType::Personal {
                    couch_filter: couch.map(|c| ObjectId::from_bytes(c)),
                });
                self.confirmed = true;
            }
            Callback::Confirm(yes) => {
                if yes {
                    ctx.ledger
                        .subscriptions
                        .edit_type(&mut ctx.session, self.id, self.tp.as_ref().unwrap())
                        .await?;
                }
                return Ok(Jmp::Back);
            }
        }
        Ok(Jmp::Stay)
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
enum Callback {
    Group(bool),
    Couch(Option<[u8; 12]>),
    Confirm(bool),
}
