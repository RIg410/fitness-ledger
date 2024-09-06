mod notifier;

use eyre::Error;
use ledger::Ledger;
use log::info;
use model::ids::DayId;
use std::time::Duration;
use tokio::time::{self};

pub fn start(ledger: Ledger) {
    tokio::spawn(async move {
        let mut interval = time::interval(Duration::from_secs(1 * 60));
        loop {
            interval.tick().await;
            if let Err(err) = process(&ledger).await {
                log::error!("Error in background process: {:#}", err);
            }
        }
    });
}

async fn process(ledger: &Ledger) -> Result<(), Error> {
    let now = chrono::Local::now();
    let mut session = ledger.db.start_session().await?;
    let mut day = ledger
        .calendar
        .get_day(&mut session, DayId::from(now))
        .await?;
    for training in &mut day.training {}

    Ok(())
}
