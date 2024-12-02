use crate::{
    callback_data::{Calldata as _, TrainingIdCallback},
    context::Context,
};
use async_trait::async_trait;
use eyre::{Error, Result};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use teloxide::types::InlineKeyboardMarkup;

use super::{Callback, Dispatch, Stage};

#[async_trait]
pub trait StageList<S>
where
    S: Send + Sync + 'static,
{
    async fn message(
        &self,
        ctx: &mut Context,
        state: &mut S,
        limit: usize,
        offset: usize,
    ) -> Result<(String, Vec<Vec<ListItem>>)>;

    fn back(&self) -> Option<Stage<S>> {
        None
    }

    async fn select(
        &self,
        ctx: &mut Context,
        state: &mut S,
        id: ListId,
    ) -> Result<Dispatch<S>, Error>;

    async fn query(&self, _: &mut Context, _: &mut S, _: &str) -> Result<(), Error> {
        Ok(())
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ListItem {
    pub id: ListId,
    pub name: String,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub enum ListId {
    I64(i64),
    ObjectId([u8; 12]),
    TrainingId(TrainingIdCallback),
    Yes,
    No,
}

impl ListId {
    pub fn as_object_id(&self) -> Option<ObjectId> {
        match self {
            ListId::ObjectId(id) => Some(ObjectId::from_bytes(*id)),
            _ => None,
        }
    }

    pub fn as_i64(&self) -> Option<i64> {
        match self {
            ListId::I64(id) => Some(*id),
            _ => None,
        }
    }
}

impl From<ObjectId> for ListId {
    fn from(id: ObjectId) -> Self {
        ListId::ObjectId(id.bytes())
    }
}

impl From<i64> for ListId {
    fn from(id: i64) -> Self {
        ListId::I64(id)
    }
}

pub struct List<S> {
    pub(super) hdl: Box<dyn StageList<S> + Send + Sync + 'static>,
    pub(super) limit: usize,
    pub(super) offset: usize,
}

impl<S> List<S>
where
    S: Send + Sync + 'static,
{
    pub(super) async fn render(
        &mut self,
        ctx: &mut Context,
        state: &mut S,
    ) -> Result<(String, InlineKeyboardMarkup)> {
        let mut keymap = InlineKeyboardMarkup::default();
        let (msg, list) = self
            .hdl
            .message(ctx, state, self.limit, self.offset)
            .await?;
        let list_len = list.len();
        for item in list {
            let mut row = Vec::new();
            for item in item {
                row.push(Callback::Select(item.id).button(item.name));
            }
            keymap = keymap.append_row(row);
        }

        let mut row = Vec::new();
        if self.offset > 0 {
            row.push(Callback::Page(-1).button("⬅️"));
        }
        if list_len == self.limit {
            row.push(Callback::Page(1).button("➡️"));
        }
        if !row.is_empty() {
            keymap = keymap.append_row(row);
        }

        Ok((msg, keymap))
    }
}
