use std::time::Duration;

use eyre::Error;
use ledger::Ledger;
use tokio::time::{self};

pub fn start(ledger: Ledger) {
    tokio::spawn(async move {
        let mut interval = time::interval(Duration::from_secs(10 * 60));
        loop {
            interval.tick().await;
            if let Err(err) = process(&ledger) {
                log::error!("Error in background process: {:#}", err);
            }
        }
    });
}

fn process(ledger: &Ledger) -> Result<(), Error> {
    Ok(())
}
