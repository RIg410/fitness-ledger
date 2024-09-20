use std::ops::{Deref, DerefMut};

use crate::{callback_data::Calldata, context::Context};
use async_trait::async_trait;
use eyre::Result;
use teloxide::types::Message;

#[async_trait]
pub trait View {
    fn allow_unsigned_user(&self) -> bool {
        false
    }

    fn can_go_back(&self) -> bool {
        true
    }

    async fn show(&mut self, ctx: &mut Context) -> Result<(), eyre::Error>;

    async fn handle_message(
        &mut self,
        ctx: &mut Context,
        msg: &Message,
    ) -> Result<Goto, eyre::Error> {
        ctx.delete_msg(msg.id).await?;
        Ok(Goto::None)
    }

    async fn handle_callback(&mut self, _: &mut Context, _: &str) -> Result<Goto, eyre::Error> {
        Ok(Goto::None)
    }

    fn widget(self) -> Widget
    where
        Self: Sized + Send + Sync + 'static,
    {
        Widget {
            view: Box::new(self),
            back: None,
        }
    }
}

pub struct Widget {
    view: Box<dyn View + Send + Sync + 'static>,
    back: Option<Box<Widget>>,
}

impl Widget {
    pub fn set_back(&mut self, back: Widget) {
        self.back = Some(Box::new(back));
    }

    pub fn take_back(&mut self) -> Option<Widget> {
        self.back.take().map(|b| *b)
    }
}

impl<T: View + Send + Sync + 'static> From<T> for Widget {
    fn from(value: T) -> Self {
        Widget {
            view: Box::new(value),
            back: None,
        }
    }
}

impl Deref for Widget {
    type Target = Box<dyn View + Send + Sync + 'static>;

    fn deref(&self) -> &Self::Target {
        &self.view
    }
}

impl DerefMut for Widget {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.view
    }
}

pub enum Goto {
    Next(Widget),
    None,
    Back,
}

impl<T: View + Send + Sync + 'static> From<T> for Goto {
    fn from(value: T) -> Self {
        Goto::Next(value.into())
    }
}

impl From<Widget> for Goto {
    fn from(value: Widget) -> Self {
        Goto::Next(value)
    }
}
