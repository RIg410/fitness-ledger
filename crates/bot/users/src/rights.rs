use super::View;
use async_trait::async_trait;
use bot_core::{callback_data::Calldata, calldata, context::Context, widget::Jmp};
use bot_viewer::user::fmt_user_type;
use model::{rights::Rule, user::User};
use serde::{Deserialize, Serialize};
use teloxide::types::InlineKeyboardMarkup;

#[derive(Default)]
pub struct UserRightsView {
    tg_id: i64,
}

impl UserRightsView {
    pub fn new(tg_id: i64) -> UserRightsView {
        UserRightsView { tg_id }
    }
}

#[async_trait]
impl View for UserRightsView {
    fn name(&self) -> &'static str {
        "UserRightsView"
    }

    async fn show(&mut self, ctx: &mut Context) -> Result<(), eyre::Error> {
        let user = ctx
            .ledger
            .users
            .get(&mut ctx.session, self.tg_id)
            .await?
            .ok_or_else(|| eyre::eyre!("Failed to load user"))?;
        let (text, markup) = render_user_rights(&user);
        ctx.edit_origin(&text, markup).await?;
        Ok(())
    }

    async fn handle_callback(&mut self, ctx: &mut Context, data: &str) -> Result<Jmp, eyre::Error> {
        let cb = calldata!(data);
        match cb {
            Callback::EditRule(rule_id, is_active) => {
                ctx.ensure(Rule::EditUserRights)?;

                let rule = Rule::try_from(rule_id)?;
                ctx.ledger
                    .users
                    .edit_user_rule(&mut ctx.session, self.tg_id, rule, is_active)
                    .await?;
                ctx.reload_user().await?;
                Ok(Jmp::Stay)
            }
        }
    }
}

fn render_user_rights(user: &User) -> (String, InlineKeyboardMarkup) {
    let mut msg = format!("{} 🔒Права:", fmt_user_type(user));
    let mut keymap = InlineKeyboardMarkup::default();

    if !user.rights.is_full() {
        for (rule, is_active) in user.rights.get_all_rules().iter() {
            keymap = keymap.append_row(Callback::EditRule(rule.id(), !is_active).btn_row(format!(
                "{} {}",
                rule.name(),
                if *is_active { "✅" } else { "❌" }
            )));
        }
    } else {
        msg.push_str("\n\nПользователь имеет права администратора");
    }

    (msg, keymap)
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Callback {
    EditRule(u8, bool),
}
