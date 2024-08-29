pub(crate) async fn handle_message(
    bot: &teloxide::Bot,
    me: &storage::user::User,
    ledger: &ledger::Ledger,
    message: &teloxide::prelude::Message,
) -> Result<Option<crate::state::State>, eyre::Error> {
    todo!()
}

pub(crate) async fn handle_callback(
    bot: &teloxide::Bot,
    me: &storage::user::User,
    ledger: &ledger::Ledger,
    q: &teloxide::prelude::CallbackQuery,
) -> Result<Option<crate::state::State>, eyre::Error> {
    todo!()
}
