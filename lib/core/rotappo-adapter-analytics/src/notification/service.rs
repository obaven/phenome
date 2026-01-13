use anyhow::Result;
use async_trait::async_trait;
use std::sync::{Arc, RwLock};

use rotappo_domain::{Notification, NotificationChannel};
use rotappo_ports::{AnalyticsPort, NotificationPort};

#[derive(Debug, Clone, Default)]
pub struct NotificationService {
    channels: Arc<RwLock<Vec<NotificationChannel>>>,
}

impl NotificationService {
    pub fn new(channels: Vec<NotificationChannel>) -> Self {
        Self {
            channels: Arc::new(RwLock::new(channels)),
        }
    }

    pub fn channels(&self) -> Vec<NotificationChannel> {
        self.channels
            .read()
            .expect("notification channels lock poisoned")
            .clone()
    }
}

#[async_trait]
impl NotificationPort for NotificationService {
    async fn send_notification(&self, _notification: Notification) -> Result<()> {
        Ok(())
    }

    async fn configure_channel(&self, channel: NotificationChannel) -> Result<()> {
        let mut channels = self
            .channels
            .write()
            .expect("notification channels lock poisoned");
        if let Some(existing) = channels
            .iter_mut()
            .find(|existing| existing.id == channel.id)
        // Match by ID
        {
            *existing = channel;
        } else {
            channels.push(channel);
        }
        Ok(())
    }
}

impl NotificationService {
    pub async fn watch_anomalies(self: Arc<Self>, service: Arc<crate::AnalyticsService>) {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(60));
        let mut last_check = chrono::Utc::now().timestamp_millis();

        loop {
            interval.tick().await;
            let now = chrono::Utc::now().timestamp_millis();

            // Query anomalies detected since last check
            let filter = rotappo_domain::AnomalyFilter {
                time_range: Some(rotappo_domain::TimeRange {
                    start_ms: last_check,
                    end_ms: now,
                }),
                ..Default::default()
            };

            if let Ok(anomalies) = service.get_anomalies(filter).await {
                for anomaly in anomalies {
                    let notification = rotappo_domain::Notification {
                        // Populate notification from anomaly
                        id: uuid::Uuid::new_v4().to_string(),
                        title: format!("Anomaly Detected: {:?}", anomaly.metric_type),
                        message: anomaly.description.clone(),
                        severity: anomaly.severity,
                        timestamp: now,
                        read: false,
                        link: None,
                        cluster_id: Some(anomaly.cluster_id.clone()),
                        resource_id: Some(anomaly.resource_id.clone()),
                    };

                    if let Err(e) = self.send_notification(notification).await {
                        tracing::error!("Failed to send anomaly notification: {}", e);
                    }
                }
            }

            last_check = now;
        }
    }
}
