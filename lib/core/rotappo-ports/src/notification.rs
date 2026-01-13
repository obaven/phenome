use anyhow::Result;
use async_trait::async_trait;

use rotappo_domain::{Notification, NotificationChannel};

#[async_trait]
pub trait NotificationPort: Send + Sync {
    async fn send_notification(&self, notification: Notification) -> Result<()>;
    async fn configure_channel(&self, channel: NotificationChannel) -> Result<()>;
}
