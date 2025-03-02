use async_trait::async_trait;
use bot_core::{
    callback_data::Calldata as _,
    calldata,
    context::Context,
    widget::{Jmp, View},
};
use bot_viewer::day::fmt_dt;
use chrono::Local;
use model::{rights::Rule, user::comments::Comment};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use teloxide::{
    types::{InlineKeyboardMarkup, Message},
    utils::markdown::escape,
};

pub struct Comments {
    user_id: ObjectId,
    index: usize,
    comments: Vec<Comment>,
}

impl Comments {
    pub fn new(user_id: ObjectId) -> Comments {
        Comments {
            user_id,
            index: 0,
            comments: vec![],
        }
    }
}

#[async_trait]
impl View for Comments {
    fn name(&self) -> &'static str {
        "Comments"
    }

    async fn show(&mut self, ctx: &mut Context) -> Result<(), eyre::Error> {
        ctx.ensure(Rule::ViewUserComments)?;

        let user = ctx
            .ledger
            .users
            .get_extension(&mut ctx.session, self.user_id)
            .await?;

        let mut message = "Комментарии\n".to_string();
        for (idx, comment) in user.comments.iter().enumerate() {
            let msg = if idx == self.index {
                format!(
                    "✅{}:\n {}\n\n",
                    fmt_dt(&comment.created_at.with_timezone(&Local)),
                    escape(&comment.text)
                )
            } else {
                format!(
                    "{}:\n {}\n\n",
                    fmt_dt(&comment.created_at.with_timezone(&Local)),
                    escape(&comment.text)
                )
            };
            message.push_str(&msg);
        }
        let mut keymap = InlineKeyboardMarkup::default();
        let mut row = vec![];
        if self.index > 0 {
            row.push(Calldata::Prev.button("⬆️"));
        }
        if self.index + 1 < user.comments.len() {
            row.push(Calldata::Next.button("⬇️"));
        }
        keymap = keymap.append_row(row);
        if !user.comments.is_empty() && ctx.has_right(Rule::EditUserComments) {
            keymap = keymap.append_row(vec![Calldata::Delete.button("❌")]);
        }
        ctx.edit_origin(&message, keymap).await?;

        self.comments = user.comments;
        Ok(())
    }

    async fn handle_message(&mut self, ctx: &mut Context, message: &Message) -> Result<Jmp, eyre::Error> {
        ctx.delete_msg(message.id).await?;
        ctx.ensure(Rule::EditUserComments)?;

        if let Some(msg) = message.text() {
            ctx.ledger
                .users
                .add_comment(&mut ctx.session, self.user_id, msg, ctx.me.id)
                .await?;
        }

        Ok(Jmp::Stay)
    }

    async fn handle_callback(&mut self, ctx: &mut Context, data: &str) -> Result<Jmp, eyre::Error> {
        let cb: Calldata = calldata!(data);
        match cb {
            Calldata::Next => {
                self.index += 1;
                if self.comments.len() <= self.index {
                    self.index = self.comments.len() - 1;
                }
            }
            Calldata::Prev => {
                self.index = self.index.saturating_sub(1);
            }
            Calldata::Delete => {
                ctx.ensure(Rule::EditUserComments)?;
                if let Some(comment) = self.comments.get(self.index) {
                    ctx.ledger
                        .users
                        .delete_comment(&mut ctx.session, self.user_id, comment.id)
                        .await?;
                }
            }
        }

        Ok(Jmp::Stay)
    }
}

#[derive(Serialize, Deserialize)]
pub enum Calldata {
    Next,
    Prev,
    Delete,
}
