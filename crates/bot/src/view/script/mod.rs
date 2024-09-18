use super::View;
use crate::{callback_data::Calldata as _, context::Context, state::Widget};
use async_trait::async_trait;
use eyre::{Error, Result};
use list::{List, ListId, StageList};
use serde::{Deserialize, Serialize};
use teloxide::types::{InlineKeyboardMarkup, Message};
use text::StageText;

pub mod list;
pub mod text;

#[async_trait]
pub trait StageYesNo<S>
where
    S: Send + Sync + 'static,
{
    async fn message(&self, ctx: &mut Context, state: &mut S) -> Result<String, Error>;
    async fn yes(&self, ctx: &mut Context, state: &mut S) -> Result<Option<Stage<S>>, Error>;
    fn back(&self) -> Option<Stage<S>> {
        None
    }
}

pub enum Stage<S> {
    Text {
        hdl: Box<dyn StageText<S> + Send + Sync + 'static>,
    },
    YesNo {
        hdl: Box<dyn StageYesNo<S> + Send + Sync + 'static>,
    },
    List(List<S>),
}

impl<S> Stage<S>
where
    S: Send + Sync + 'static,
{
    pub fn text(hdl: impl StageText<S> + Send + Sync + 'static) -> Stage<S> {
        Stage::Text { hdl: Box::new(hdl) }
    }

    pub fn yes_no(hdl: impl StageYesNo<S> + Send + Sync + 'static) -> Stage<S> {
        Stage::YesNo { hdl: Box::new(hdl) }
    }

    pub fn list(hdl: impl StageList<S> + Send + Sync + 'static) -> Stage<S> {
        Stage::List(List {
            hdl: Box::new(hdl),
            limit: 7,
            offset: 0,
        })
    }

    fn back(&self) -> Option<Stage<S>> {
        match self {
            Stage::List(list) => list.hdl.back(),
            Stage::YesNo { hdl } => hdl.back(),
            Stage::Text { hdl } => hdl.back(),
        }
    }
}

pub struct ScriptView<S> {
    state: Option<S>,
    action: Option<Stage<S>>,
    go_back: Option<Widget>,
}

impl<S> ScriptView<S>
where
    S: Send + Sync + 'static,
{
    pub fn new(state: S, action: Stage<S>, go_back: Widget) -> ScriptView<S> {
        ScriptView {
            state: Some(state),
            action: Some(action),
            go_back: Some(go_back),
        }
    }
}

#[async_trait]
impl<S> View for ScriptView<S>
where
    S: Send + Sync + 'static,
{
    async fn show(&mut self, ctx: &mut Context) -> Result<()> {
        if let Some(stage) = self.action.as_mut() {
            let mut keymap = InlineKeyboardMarkup::default();

            let (msg, mut keymap) = match stage {
                Stage::Text { hdl, .. } => (
                    hdl.message(ctx, self.state.as_mut().unwrap()).await?,
                    keymap,
                ),
                Stage::YesNo { hdl, .. } => {
                    keymap = keymap.append_row(vec![
                        Callback::Select(ListId::Yes).button("✅Да"),
                        Callback::Select(ListId::No).button("❌Нет"),
                    ]);
                    (
                        hdl.message(ctx, self.state.as_mut().unwrap()).await?,
                        keymap,
                    )
                }
                Stage::List(list) => list.render(ctx, self.state.as_mut().unwrap()).await?,
            };
            keymap = keymap.append_row(Callback::Back.btn_row("🔙 Назад"));
            ctx.edit_origin(&msg, keymap).await?;
        }
        Ok(())
    }

    async fn handle_message(
        &mut self,
        ctx: &mut Context,
        message: &Message,
    ) -> Result<Option<Widget>> {
        let (action, text) =
            if let (Some(action), Some(text)) = (self.action.as_mut(), message.text()) {
                (action, text)
            } else {
                return Ok(None);
            };

        match action {
            Stage::Text { hdl } => {
                let mut next = hdl
                    .handle_text(ctx, self.state.as_mut().unwrap(), text)
                    .await?;
                if let Some(next) = next.take() {
                    self.action = Some(next);
                }
                self.show(ctx).await?;
            }
            Stage::YesNo { .. } => {}
            Stage::List(list) => {
                list.hdl
                    .query(ctx, self.state.as_mut().unwrap(), text)
                    .await?;
                self.show(ctx).await?;
            }
        }
        ctx.delete_msg(message.id).await?;
        Ok(None)
    }

    async fn handle_callback(&mut self, ctx: &mut Context, data: &str) -> Result<Option<Widget>> {
        let (action, cb) =
            if let (Some(action), Some(cb)) = (self.action.as_mut(), Callback::from_data(data)) {
                (action, cb)
            } else {
                return Ok(None);
            };

        let next = match cb {
            Callback::Back => {
                if let Some(back) = action.back() {
                    self.action = Some(back);
                    self.show(ctx).await?;
                    return Ok(None);
                } else {
                    return Ok(self.go_back.take());
                }
            }
            Callback::Select(idx) => match action {
                Stage::Text { .. } => {
                    return Ok(None);
                }
                Stage::YesNo { hdl, .. } => match idx {
                    ListId::Yes => {
                        if let Some(next) = hdl.yes(ctx, self.state.as_mut().unwrap()).await? {
                            Some(next)
                        } else {
                            return Ok(self.go_back.take());
                        }
                    }
                    ListId::No => {
                        ctx.send_notification("❌ Отменено").await?;
                        return Ok(self.go_back.take());
                    }
                    _ => return Ok(None),
                },
                Stage::List(list) => {
                    list.hdl
                        .select(ctx, self.state.as_mut().unwrap(), idx)
                        .await?
                }
            },
            Callback::Page(offset) => match action {
                Stage::List(list) => {
                    list.offset += offset as usize * list.limit;
                    self.show(ctx).await?;
                    return Ok(None);
                }
                _ => return Ok(None),
            },
        };
        if let Some(next) = next {
            self.action = Some(next);
        }

        self.show(ctx).await?;
        Ok(None)
    }

    fn take(&mut self) -> Widget
    where
        Self: Sized + Send + Sync + 'static,
    {
        ScriptView {
            state: self.state.take(),
            action: self.action.take(),
            go_back: self.go_back.take(),
        }
        .boxed()
    }
}

#[derive(Serialize, Deserialize, Clone)]
enum Callback {
    Back,
    Select(ListId),
    Page(i8),
}
