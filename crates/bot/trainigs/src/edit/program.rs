use async_trait::async_trait;
use bot_core::{callback_data::Calldata as _, context::Context, widget::{Jmp, View}};
use model::training::TrainingId;
use serde::{Deserialize, Serialize};
use teloxide::types::InlineKeyboardMarkup;

pub struct ChangeProgram {
    id: TrainingId,
    all: bool,
}

impl ChangeProgram {
    pub fn new(id: TrainingId, all: bool) -> Self {
        Self { id, all }
    }
}

#[async_trait]
impl View for ChangeProgram {
    fn name(&self) -> &'static str {
        "Change program"
    }

    async fn show(&mut self, ctx: &mut Context) -> Result<(), eyre::Error> {
        let msg = "Тренировочные программы: 🤸🏼".to_string();
        let mut keymap = InlineKeyboardMarkup::default();

        let trainings = ctx.ledger.programs.get_all(&mut ctx.session, false).await?;

        for training in trainings {
            let name = if training.visible {
                training.name.clone()
            } else {
                format!("🔒 {}", training.name)
            };
            keymap
                .inline_keyboard
                .push(Callback::SelectTraining(training.id.bytes()).btn_row(name));
        }

        Ok(())
    }

    async fn handle_callback(&mut self, ctx: &mut Context, data: &str) -> Result<Jmp, eyre::Error> {
        
        Ok(Jmp::Stay)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum Callback {
    SelectTraining([u8; 12]),
}

