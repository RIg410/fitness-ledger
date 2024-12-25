use async_trait::async_trait;
use bot_core::{
    callback_data::Calldata as _,
    calldata,
    context::Context,
    widget::{Jmp, View},
};
use eyre::Error;
use model::rights::Rule;
use serde::{Deserialize, Serialize};
use teloxide::types::{InlineKeyboardMarkup, Message};

mod subscription;

#[derive(Default)]
pub struct SystemView {}

impl SystemView {}

#[async_trait]
impl View for SystemView {
    fn name(&self) -> &'static str {
        "SystemView"
    }

    async fn show(&mut self, ctx: &mut Context) -> Result<(), Error> {
        ctx.ensure(Rule::System)?;
        let mut keymap = InlineKeyboardMarkup::default();
        keymap = keymap.append_row((Calldata::Dump).btn_row("🗑️ Dump"));
        keymap = keymap.append_row((Calldata::ApplyDump).btn_row("🔄 ApplyDump"));
        keymap =
            keymap.append_row((Calldata::ExtendSubscription).btn_row("🔄 Extend subscription"));
        ctx.edit_origin("🔧System", keymap).await?;
        Ok(())
    }

    async fn handle_callback(&mut self, ctx: &mut Context, data: &str) -> Result<Jmp, Error> {
        ctx.ensure(Rule::System)?;
        match calldata!(data) {
            Calldata::Dump => {
                let dump_file = ctx.ledger.backup.make_backup(&mut ctx.session).await?;
                ctx.send_document(dump_file, "dump.zip").await?;
            }
            Calldata::ExtendSubscription => {
                return Ok(subscription::ExtendSubscriptions.into());
            }
            Calldata::ApplyDump => {
                return Ok(ApplyDump.into());
            }
        }
        Ok(Jmp::Stay)
    }
}

#[derive(Serialize, Deserialize)]
enum Calldata {
    Dump,
    ApplyDump,
    ExtendSubscription,
}

pub struct ApplyDump;

#[async_trait]
impl View for ApplyDump {
    fn name(&self) -> &'static str {
        "ApplyDump"
    }

    async fn show(&mut self, ctx: &mut Context) -> Result<(), Error> {
        ctx.ensure(Rule::System)?;
        ctx.edit_origin("Отправьте дамп", Default::default())
            .await?;
        Ok(())
    }

    async fn handle_message(&mut self, ctx: &mut Context, msg: &Message) -> Result<Jmp, Error> {
        ctx.ensure(Rule::System)?;
        log::info!("Apply dump");
        if let Some(document) = msg.document() {
            log::info!("Apply dump {:?}", document);
            let dump = ctx.bot.load_document(&document.file).await?;
            ctx.ledger
                .backup
                .apply_backup(&mut ctx.session, dump)
                .await?;
            ctx.send_msg("Дамп применен").await?;
            Ok(Jmp::Stay)
        } else {
            ctx.send_msg("Отправьте дамп").await?;
            return Ok(Jmp::Stay);
        }
    }
}
