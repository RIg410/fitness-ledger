use super::View;
use crate::{callback_data::Calldata as _, context::Context, state::Widget};
use async_trait::async_trait;
use chrono::Local;
use eyre::{Error, Result};
use model::log::LogEntry;
use serde::{Deserialize, Serialize};
use teloxide::{types::{InlineKeyboardMarkup, Message}, utils::markdown::escape};

const PAGE_SIZE: usize = 5;

#[derive(Default)]
pub struct LogsView {
    offset: usize,
}

impl LogsView {
    async fn render_log(&self, msg: &mut String, entry: LogEntry) -> Result<(), Error> {
        let date = entry
            .date_time
            .with_timezone(&Local)
            .format("%Y-%m-%d %H:%M:%S");
        msg.push_str(&escape(&format!("{}: {:?}\n\n", date, entry.action)));
        Ok(())
    }
}

#[async_trait]
impl View for LogsView {
    async fn show(&mut self, ctx: &mut Context) -> Result<()> {
        let mut message = format!("Logs\n");
        let logs = ctx
            .ledger
            .logs
            .logs(&mut ctx.session, PAGE_SIZE, self.offset)
            .await?;

        for log in logs {
            self.render_log(&mut message, log).await?;
        }

        let keymap = vec![vec![
            Calldata::Back.button("⬅️ Back"),
            Calldata::Forward.button("➡️ Forward"),
        ]];
        ctx.edit_origin(&message, InlineKeyboardMarkup::new(keymap))
            .await?;
        Ok(())
    }

    async fn handle_message(
        &mut self,
        ctx: &mut Context,
        message: &Message,
    ) -> Result<Option<Widget>> {
        ctx.delete_msg(message.id).await?;
        Ok(None)
    }

    async fn handle_callback(&mut self, ctx: &mut Context, data: &str) -> Result<Option<Widget>> {
        let data = if let Some(data) = Calldata::from_data(data) {
            data
        } else {
            return Ok(None);
        };
        match data {
            Calldata::Back => {
                self.offset = self.offset.saturating_sub(PAGE_SIZE);
            }
            Calldata::Forward => {
                self.offset += PAGE_SIZE;
            }
        }
        self.show(ctx).await?;
        Ok(None)
    }
}

#[derive(Serialize, Deserialize)]
pub enum Calldata {
    Back,
    Forward,
}
