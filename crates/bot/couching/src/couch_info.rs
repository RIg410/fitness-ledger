use async_trait::async_trait;
use bot_core::{
    context::Context, script::{list::{ListId, ListItem, StageList}, Dispatch, ScriptView, Stage}, widget::Widget
};
use eyre::{Error, Result};
use mongodb::bson::oid::ObjectId;

pub fn couch_view(id: ObjectId) -> Widget {
    ScriptView::new("couch_info", State { id }, Stage::list(CouchInfo)).into()
}

struct State {
    id: ObjectId,
}

struct CouchInfo;

#[async_trait]
impl StageList<State> for CouchInfo {
    async fn message(
        &self,
        ctx: &mut Context,
        state: &mut State,
        limit: usize,
        offset: usize,
    ) -> Result<(String, Vec<Vec<ListItem>>)> {
        todo!()
    }

    async fn select(
        &self,
        ctx: &mut Context,
        state: &mut State,
        id: ListId,
    ) -> Result<Dispatch<State>, Error> {
        todo!()
    }
}

