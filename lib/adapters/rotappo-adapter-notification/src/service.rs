use anyhow::Result;
use phenome_domain::notification::{Notification, NotificationChannel};
use phenome_ports::NotificationPort;

#[derive(Debug)]
pub struct NotificationService;

#[tonic::async_trait]
impl NotificationPort for NotificationService {
    async fn send_notification(&self, notification: Notification) -> Result<()> {
        println!("Sending notification: {:?}", notification);
        Ok(())
    }

    async fn configure_channel(&self, channel: NotificationChannel) -> Result<()> {
        println!("Configuring channel: {:?}", channel);
        Ok(())
    }
}
