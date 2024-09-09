use eyre::Error;
use ledger::Ledger;
use std::time::Duration;
use tokio::time::{self};

pub fn start(ledger: Ledger) {
    tokio::spawn(async move {
        let mut interval = time::interval(Duration::from_secs(5 * 60));
        loop {
            interval.tick().await;
            if let Err(err) = process(&ledger).await {
                log::error!("Error in background process: {:#}", err);
            }
        }
    });
}

async fn process(ledger: &Ledger) -> Result<(), Error> {
    let mut session = ledger.db.start_session().await?;
    ledger.process(&mut session).await?;
    Ok(())
}
