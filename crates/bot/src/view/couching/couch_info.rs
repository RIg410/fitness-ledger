use crate::{
    context::Context,
    state::Widget,
    view::{
        script::{
            list::{ListId, ListItem, StageList},
            ScriptView, Stage,
        },
        View as _,
    },
};
use async_trait::async_trait;
use eyre::{Error, Result};
use mongodb::bson::oid::ObjectId;

pub fn couch_view(go_back: Widget, id: ObjectId) -> Widget {
    ScriptView::new(State { id }, Stage::list(CouchInfo), go_back).boxed()
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
    ) -> Result<Option<Stage<State>>, Error> {
        todo!()
    }
}
