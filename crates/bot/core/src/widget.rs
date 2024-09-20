use std::ops::{Deref, DerefMut};

use crate::context::Context;
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
    ) -> Result<Jmp, eyre::Error> {
        ctx.delete_msg(msg.id).await?;
        Ok(Jmp::None)
    }

    async fn handle_callback(&mut self, _: &mut Context, _: &str) -> Result<Jmp, eyre::Error> {
        Ok(Jmp::None)
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

    pub fn has_back(&self) -> bool {
        self.back.is_some()
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

pub enum Jmp {
    Next(Widget),
    Goto(Widget),
    None,
    Back,
    Home,
}

impl<T: View + Send + Sync + 'static> From<T> for Jmp {
    fn from(value: T) -> Self {
        Jmp::Next(value.into())
    }
}

impl From<Widget> for Jmp {
    fn from(value: Widget) -> Self {
        Jmp::Next(value)
    }
}
