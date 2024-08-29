use eyre::Result;
use ledger::Ledger;
use storage::user::User;
use teloxide::{
    types::{ChatId, MessageId},
    Bot,
};

use crate::{process::users_menu::{search::Query, SelectedUser}, state::State};

pub async fn show_user_rights(
    bot: &Bot,
    me: &User,
    ledger: &Ledger,
    user_id: String,
    chat_id: ChatId,
    query: SelectedUser,
) -> Result<Option<State>> {
    todo!()
}
