use crate::Ledger;
use chrono::{Duration, Local};
use eyre::Result;
use log::info;
use model::session::Session;

pub struct LogsBg {
    ledger: Ledger,
}

impl LogsBg {
    pub fn new(ledger: Ledger) -> LogsBg {
        LogsBg { ledger }
    }

    pub async fn process(&self, session: &mut Session) -> Result<()> {
        let dt = Local::now() - Duration::days(30);
        let count = self.ledger.logs.gc(session, dt).await?;
        if count > 0 {
            info!("Deleted logs entrees:{}", count);
        }
        Ok(())
    }
}
