use ai::AiModel;
use async_trait::async_trait;
use bot_core::{
    callback_data::Calldata as _,
    calldata,
    context::Context,
    widget::{Jmp, View},
};
use model::rights::Rule;
use serde::{Deserialize, Serialize};
use teloxide::{
    types::{InlineKeyboardButton, InlineKeyboardMarkup, Message},
};

pub struct AiView {
    model: AiModel,
}

impl AiView {
    pub fn new(model: AiModel) -> Self {
        Self { model }
    }
}

#[async_trait]
impl View for AiView {
    fn name(&self) -> &'static str {
        "AiView"
    }

    async fn show(&mut self, ctx: &mut Context) -> Result<(), eyre::Error> {
        let msg = "Я ваш помощник по маркетингу\\. Чем могу помочь?";
        let mut keymap = InlineKeyboardMarkup::default();
        if ctx.has_right(Rule::SelectModel) {
            keymap = keymap.append_row(vec![
                model_btn(AiModel::Gpt4oMini, self.model),
                model_btn(AiModel::Gpt4o, self.model),
            ]);
            keymap = keymap.append_row(vec![
                model_btn(AiModel::Claude3Haiku, self.model),
                model_btn(AiModel::Claude3Opus, self.model),
                model_btn(AiModel::Claude3Sonnet, self.model),
            ]);
        }
        ctx.edit_origin(msg, keymap).await?;
        Ok(())
    }

    async fn handle_message(
        &mut self,
        ctx: &mut Context,
        msg: &Message,
    ) -> Result<Jmp, eyre::Error> {
        let text = msg.text().unwrap_or_default();

        ctx.send_notification("Думаю\\.\\.\\.").await;
        let ai_response = ctx
            .ledger
            .statistics
            .ask_ai(&mut ctx.session, self.model, text.to_string())
            .await;
        match ai_response {
            Ok(resp) => {
                ctx.send_notification(&resp).await;
            }
            Err(err) => {
                ctx.send_notification(&format!("Ошибка: {}", err)).await;
            }
        }
        Ok(Jmp::Stay)
    }

    async fn handle_callback(&mut self, _: &mut Context, data: &str) -> Result<Jmp, eyre::Error> {
        match calldata!(data) {
            Callback::SetModel(model) => {
                self.model = model;
            }
        }
        Ok(Jmp::Stay)
    }
}

#[derive(Serialize, Deserialize)]
enum Callback {
    SetModel(AiModel),
}

fn model_btn(model: AiModel, selected: AiModel) -> InlineKeyboardButton {
    let selected = if model == selected { "✅" } else { "" };
    Callback::SetModel(model).button(format!("{}{}", selected, model.name()))
}
