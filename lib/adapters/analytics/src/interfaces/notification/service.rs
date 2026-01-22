use anyhow::Result;
use async_trait::async_trait;
use std::sync::{Arc, RwLock};
use std::time::Duration;
use tokio::sync::watch;
use tokio::time::interval;

use phenome_domain::{Notification, NotificationChannel};
use phenome_ports::{AnalyticsPort, NotificationPort};

const ANOMALY_POLL_INTERVAL: Duration = Duration::from_secs(60);
const MAX_ANOMALIES_PER_TICK: usize = 50;

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
        match self.channels.read() {
            Ok(guard) => guard.clone(),
            Err(_) => {
                tracing::error!("notification channels lock poisoned");
                Vec::new()
            }
        }
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
            .map_err(|_| anyhow::anyhow!("notification channels lock poisoned"))?;
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
        let (_tx, rx) = watch::channel(false);
        self.watch_anomalies_with_shutdown(service, rx).await;
    }

    pub async fn watch_anomalies_with_shutdown(
        self: Arc<Self>,
        service: Arc<crate::AnalyticsService>,
        mut shutdown: watch::Receiver<bool>,
    ) {
        let mut interval = interval(ANOMALY_POLL_INTERVAL);
        let mut last_check = chrono::Utc::now().timestamp_millis();

        loop {
            tokio::select! {
                result = shutdown.changed() => {
                    if result.is_err() || *shutdown.borrow() {
                        break;
                    }
                }
                _ = interval.tick() => {
                    let now = chrono::Utc::now().timestamp_millis();

                    // Query anomalies detected since last check
                    let filter = phenome_domain::AnomalyFilter {
                        time_range: Some(phenome_domain::TimeRange {
                            start_ms: last_check,
                            end_ms: now,
                        }),
                        ..Default::default()
                    };

                    match service.get_anomalies(filter).await {
                        Ok(mut anomalies) => {
                            if anomalies.len() > MAX_ANOMALIES_PER_TICK {
                                tracing::warn!(
                                    "Anomaly notifications capped at {} per tick",
                                    MAX_ANOMALIES_PER_TICK
                                );
                                anomalies.truncate(MAX_ANOMALIES_PER_TICK);
                            }

                            for anomaly in anomalies {
                                let notification = phenome_domain::Notification {
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
                        Err(err) => {
                            tracing::error!("Failed to query anomalies: {}", err);
                        }
                    }

                    last_check = now;
                }
            }
        }
    }
}
