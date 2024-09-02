use async_trait::async_trait;
use eyre::Error;

#[async_trait]
pub trait Notifier {
    fn notify(&self, tg_id: i64, message: &str) -> Result<(), Error>;
}
