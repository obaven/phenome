use anyhow::Result;
use async_trait::async_trait;
use notify_rust::Notification as SystemNotification;
use reqwest::Client;
use phenome_domain::{Notification, NotificationChannel};
use phenome_ports::NotificationPort;
use std::sync::{Arc, RwLock};

#[derive(Clone)]
pub struct NotificationService {
    channels: Arc<RwLock<Vec<NotificationChannel>>>,
    http_client: Client,
}

impl NotificationService {
    pub fn new() -> Self {
        Self {
            channels: Arc::new(RwLock::new(Vec::new())),
            http_client: Client::new(),
        }
    }
}

#[async_trait]
impl NotificationPort for NotificationService {
    async fn send_notification(&self, notification: Notification) -> Result<()> {
        let channels = {
            let guard = self
                .channels
                .read()
                .map_err(|_| anyhow::anyhow!("lock poisoned"))?;
            guard.clone()
        };

        for channel in channels {
            match channel {
                NotificationChannel::System => {
                    let _ = SystemNotification::new()
                        .summary(&notification.title)
                        .body(&notification.message)
                        .show(); // Note: show() is blocking or async depending on feature, here likely blocking but fast enough or should spawn
                }
                NotificationChannel::InTui => {
                    // In-TUI delivery currently emits to tracing; UI can consume via log stream.
                    tracing::info!("InTui notification: {:?}", notification);
                }
                NotificationChannel::Ntfy { url, topic } => {
                    let full_url = format!("{}/{}", url.trim_end_matches('/'), topic);
                    let _ = self
                        .http_client
                        .post(&full_url)
                        .body(notification.message.clone())
                        .header("Title", notification.title.clone())
                        .send()
                        .await;
                }
                NotificationChannel::Webhook { url, headers } => {
                    let mut req = self.http_client.post(&url).json(&notification);
                    for (k, v) in headers {
                        req = req.header(k, v);
                    }
                    let _ = req.send().await;
                }
            }
        }
        Ok(())
    }

    async fn configure_channel(&self, channel: NotificationChannel) -> Result<()> {
        let mut channels = self
            .channels
            .write()
            .map_err(|_| anyhow::anyhow!("lock poisoned"))?;
        // Simple append for now. Deduplication logic might be needed.
        channels.push(channel);
        Ok(())
    }
}
