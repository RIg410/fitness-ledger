use bot_main::BotApp;
use eyre::Error;
use ledger::Ledger;
use process::BgProcessor;
use std::time::Duration;
use tokio::time::{self};
mod process;

pub fn start(ledger: Ledger, bot: BotApp) {
    let bg_process = BgProcessor::new(ledger, bot);
    tokio::spawn(async move {
        let mut interval = time::interval(Duration::from_secs(10 * 60));
        loop {
            interval.tick().await;
            if let Err(err) = process(&bg_process).await {
                log::error!("Error in background process: {:#}", err);
            }
        }
    });
}

async fn process(proc: &BgProcessor) -> Result<(), Error> {
    let mut session = proc.ledger.db.start_session().await?;
    proc.process(&mut session).await?;
    Ok(())
}
