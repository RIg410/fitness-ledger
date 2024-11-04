use std::sync::Arc;

use env::Env;
use eyre::Context;
use log::info;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    let env = Env::load()?;

    pretty_env_logger::init();
    color_eyre::install()?;
    info!("connecting to mongo");
    let storage = storage::Storage::new(env.mongo_url())
        .await
        .context("Failed to create storage")?;
    info!("creating ledger");
    let ledger = Arc::new(ledger::Ledger::new(storage, env.clone()));
    info!("Starting bot...");
    let bot: bot_main::BotApp = bot_main::BotApp::new(env);
    info!("Starting mini app...");
    mini_app_main::spawn(ledger.clone(), bot.clone())?;

    info!("Starting background process...");
    bg_process::start(ledger.clone(), bot.clone()).await?;
    info!("Starting bot...");
    bot.start(ledger).await?;

    Ok(())
}
