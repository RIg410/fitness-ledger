use eyre::Result;
use std::sync::Arc;
use storage::notification::NotificationStore;

pub struct NotificationService {
    store: Arc<NotificationStore>,
}

impl NotificationService {
    pub async fn new(store: Arc<NotificationStore>) -> Result<Self> {
        Ok(NotificationService { store })
    }
}
