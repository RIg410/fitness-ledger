use std::env;

use dotenv::dotenv;
use eyre::Context;
use log::info;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    if let Err(err) = dotenv() {
        info!("Failed to load .env file: {}", err);
    }
    pretty_env_logger::init();
    color_eyre::install()?;
    info!("connecting to mongo");
    let mongo_url = env::var("MONGO_URL").context("Failed to get MONGO_URL from env")?;
    let storage = storage::Storage::new(&mongo_url)
        .await
        .context("Failed to create storage")?;
    info!("creating ledger");
    let ledger = ledger::Ledger::new(storage);

    let token = env::var("TG_TOKEN").context("Failed to get TG_TOKEN from env")?;
    info!("Starting bot...");
    bot::start_bot(ledger.clone(), token).await?;

    Ok(())
}
