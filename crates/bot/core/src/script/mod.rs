use crate::{
    callback_data::Calldata as _,
    context::Context,
    widget::{self, Jmp, View, Widget},
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
    name: &'static str,
    state: Option<S>,
    action: Option<Stage<S>>,
}

impl<S> ScriptView<S>
where
    S: Send + Sync + 'static,
{
    pub fn new(name: &'static str, state: S, action: Stage<S>) -> ScriptView<S> {
        ScriptView {
            name,
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
    fn name(&self) -> &'static str {
        self.name
    }

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

    async fn handle_message(&mut self, ctx: &mut Context, message: &Message) -> Result<Jmp> {
        let (action, text) =
            if let (Some(action), Some(text)) = (self.action.as_mut(), message.text()) {
                (action, text)
            } else {
                return Ok(Jmp::Stay);
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
                        return Ok(Jmp::Next(view));
                    }
                    Dispatch::WidgetBack => {
                        return Ok(Jmp::Back);
                    }
                }
            }
            Stage::YesNo { .. } => {}
            Stage::List(list) => {
                list.hdl
                    .query(ctx, self.state.as_mut().unwrap(), text)
                    .await?;
            }
        }
        ctx.delete_msg(message.id).await?;
        Ok(Jmp::Stay)
    }

    async fn handle_callback(&mut self, ctx: &mut Context, data: &str) -> Result<Jmp> {
        let (action, cb) =
            if let (Some(action), Some(cb)) = (self.action.as_mut(), Callback::from_data(data)) {
                (action, cb)
            } else {
                return Ok(widget::Jmp::Stay);
            };

        match cb {
            Callback::Back => match action.back() {
                Some(back) => {
                    self.action = Some(back);
                }
                None => {
                    return Ok(Jmp::Back);
                }
            },
            Callback::Select(idx) => match action {
                Stage::Text(_) => {
                    return Ok(Jmp::Stay);
                }
                Stage::YesNo(hdl) => match idx {
                    ListId::Yes => match hdl.yes(ctx, self.state.as_mut().unwrap()).await? {
                        Dispatch::None => {
                            return Ok(Jmp::Stay);
                        }
                        Dispatch::Stage(stage) => {
                            self.action = Some(stage);
                        }
                        Dispatch::Widget(widget) => {
                            return Ok(Jmp::Next(widget));
                        }
                        Dispatch::WidgetBack => {
                            return Ok(Jmp::Back);
                        }
                    },
                    ListId::No => {
                        ctx.send_notification("❌ Отменено").await?;
                        return Ok(Jmp::Back);
                    }
                    _ => return Ok(Jmp::Stay),
                },
                Stage::List(list) => {
                    let result = list
                        .hdl
                        .select(ctx, self.state.as_mut().unwrap(), idx)
                        .await?;
                    match result {
                        Dispatch::None => {
                            return Ok(Jmp::Stay);
                        }
                        Dispatch::Stage(stage) => {
                            self.action = Some(stage);
                        }
                        Dispatch::Widget(widget) => {
                            return Ok(Jmp::Next(widget));
                        }
                        Dispatch::WidgetBack => {
                            return Ok(Jmp::Back);
                        }
                    }
                }
            },
            Callback::Page(offset) => match action {
                Stage::List(list) => {
                    if offset > 0 {
                        list.offset += offset as usize * list.limit;
                    } else {
                        list.offset -= offset.abs() as usize * list.limit;
                    }
                }
                _ => return Ok(Jmp::Stay),
            },
        };

        Ok(Jmp::Stay)
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
    WidgetBack,
}
