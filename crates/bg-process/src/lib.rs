use eyre::Error;
use ledger::{process::BgProcessor, Ledger};
use std::time::Duration;
use tokio::time::{self};

pub fn start(ledger: Ledger) {
    tokio::spawn(async move {
        let mut interval = time::interval(Duration::from_secs(5 * 60));
        let bg_process = BgProcessor::new(ledger);
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
