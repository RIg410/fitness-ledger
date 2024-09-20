use crate::{
    callback_data::Calldata as _,
    context::Context,
    widget::{self, Dest, View, Widget},
};
use async_trait::async_trait;
use eyre::Result;
use list::{List, ListId, StageList};
use serde::{Deserialize, Serialize};
use teloxide::types::{InlineKeyboardMarkup, Message};
use text::StageText;
use yes_no::StageYesNo;

pub mod list;
pub mod text;
pub mod yes_no;

pub enum Stage<S> {
    Text(Box<dyn StageText<S> + Send + Sync + 'static>),
    YesNo(Box<dyn StageYesNo<S> + Send + Sync + 'static>),
    List(List<S>),
}

impl<S> Stage<S>
where
    S: Send + Sync + 'static,
{
    pub fn text(hdl: impl StageText<S> + Send + Sync + 'static) -> Stage<S> {
        Stage::Text(Box::new(hdl))
    }

    pub fn yes_no(hdl: impl StageYesNo<S> + Send + Sync + 'static) -> Stage<S> {
        Stage::YesNo(Box::new(hdl))
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
            Stage::YesNo(hdl) => hdl.back(),
            Stage::Text(hdl) => hdl.back(),
        }
    }
}

pub struct ScriptView<S> {
    state: Option<S>,
    action: Option<Stage<S>>,
}

impl<S> ScriptView<S>
where
    S: Send + Sync + 'static,
{
    pub fn new(state: S, action: Stage<S>) -> ScriptView<S> {
        ScriptView {
            state: Some(state),
            action: Some(action),
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
                Stage::Text(hdl) => (
                    hdl.message(ctx, self.state.as_mut().unwrap()).await?,
                    keymap,
                ),
                Stage::YesNo(hdl) => {
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

            ctx.system_go_back = false;
            keymap = keymap.append_row(Callback::Back.btn_row("🔙 Назад"));
            ctx.edit_origin(&msg, keymap).await?;
        }
        Ok(())
    }

    async fn handle_message(&mut self, ctx: &mut Context, message: &Message) -> Result<Dest> {
        let (action, text) =
            if let (Some(action), Some(text)) = (self.action.as_mut(), message.text()) {
                (action, text)
            } else {
                return Ok(Dest::None);
            };

        match action {
            Stage::Text(hdl) => {
                let next = hdl
                    .handle_text(ctx, self.state.as_mut().unwrap(), text)
                    .await?;
                match next {
                    Dispatch::None => {}
                    Dispatch::Stage(stage) => self.action = Some(stage),
                    Dispatch::Widget(view) => {
                        return Ok(Dest::Next(view));
                    }
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
        Ok(Dest::None)
    }

    async fn handle_callback(&mut self, ctx: &mut Context, data: &str) -> Result<Dest> {
        let (action, cb) =
            if let (Some(action), Some(cb)) = (self.action.as_mut(), Callback::from_data(data)) {
                (action, cb)
            } else {
                return Ok(widget::Dest::None);
            };

        match cb {
            Callback::Back => match action.back() {
                Some(back) => {
                    self.action = Some(back);
                }
                None => {
                    return Ok(Dest::Back);
                }
            },
            Callback::Select(idx) => match action {
                Stage::Text(_) => {
                    return Ok(Dest::None);
                }
                Stage::YesNo(hdl) => match idx {
                    ListId::Yes => match hdl.yes(ctx, self.state.as_mut().unwrap()).await? {
                        Dispatch::None => {
                            return Ok(Dest::None);
                        }
                        Dispatch::Stage(stage) => {
                            self.action = Some(stage);
                        }
                        Dispatch::Widget(widget) => {
                            return Ok(Dest::Next(widget));
                        }
                    },
                    ListId::No => {
                        ctx.send_notification("❌ Отменено").await?;
                        return Ok(Dest::Back);
                    }
                    _ => return Ok(Dest::None),
                },
                Stage::List(list) => {
                    let result = list
                        .hdl
                        .select(ctx, self.state.as_mut().unwrap(), idx)
                        .await?;
                    match result {
                        Dispatch::None => {
                            return Ok(Dest::None);
                        }
                        Dispatch::Stage(stage) => {
                            self.action = Some(stage);
                        }
                        Dispatch::Widget(widget) => {
                            return Ok(Dest::Next(widget));
                        }
                    }
                }
            },
            Callback::Page(offset) => match action {
                Stage::List(list) => {
                    list.offset += offset as usize * list.limit;
                    self.show(ctx).await?;
                }
                _ => return Ok(Dest::None),
            },
        };

        self.show(ctx).await?;
        Ok(Dest::None)
    }
}

#[derive(Serialize, Deserialize, Clone)]
enum Callback {
    Back,
    Select(ListId),
    Page(i8),
}

pub enum Dispatch<S> {
    None,
    Stage(Stage<S>),
    Widget(Widget),
}
