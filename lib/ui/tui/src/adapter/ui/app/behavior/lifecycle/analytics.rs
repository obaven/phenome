use std::time::{Duration, Instant};

use crate::app::App;

const ANALYTICS_POLL_INTERVAL: Duration = Duration::from_secs(5);
const ANALYTICS_MAX_UPDATES_PER_TICK: usize = 32;

impl App {
    pub(super) fn start_analytics(&mut self) {
        if let Ok(client) = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current()
                .block_on(crate::analytics_client::AnalyticsClient::connect_from_env())
        }) {
            let client = client.clone();
            self.analytics_client = Some(client.clone());
            let (tx, rx) = tokio::sync::mpsc::channel(10);
            self.analytics_rx = Some(rx);

            tokio::spawn(async move {
                let mut tick = tokio::time::interval(ANALYTICS_POLL_INTERVAL);
                loop {
                    if tx.is_closed() {
                        break;
                    }
                    if let Ok(metrics) = client.fetch_metrics().await {
                        if tx
                            .send(crate::app::core::AnalyticsUpdate::Metrics(metrics))
                            .await
                            .is_err()
                        {
                            break;
                        }
                    }
                    if let Ok(anomalies) = client.fetch_anomalies().await {
                        if tx
                            .send(crate::app::core::AnalyticsUpdate::Anomalies(anomalies))
                            .await
                            .is_err()
                        {
                            break;
                        }
                    }
                    if let Ok(recs) = client.fetch_recommendations().await {
                        if tx
                            .send(crate::app::core::AnalyticsUpdate::Recommendations(recs))
                            .await
                            .is_err()
                        {
                            break;
                        }
                    }
                    tick.tick().await;
                }
            });
        }
    }

    pub(super) fn refresh_analytics_cache(&mut self) {
        if let Some(rx) = &mut self.analytics_rx {
            let mut drained = 0usize;
            while drained < ANALYTICS_MAX_UPDATES_PER_TICK {
                let update = match rx.try_recv() {
                    Ok(update) => update,
                    Err(_) => break,
                };
                match update {
                    crate::app::core::AnalyticsUpdate::Metrics(m) => {
                        self.analytics_metrics = Some(m)
                    }
                    crate::app::core::AnalyticsUpdate::Anomalies(a) => {
                        self.analytics_anomalies = Some(a)
                    }
                    crate::app::core::AnalyticsUpdate::Recommendations(r) => {
                        self.analytics_recommendations = Some(r)
                    }
                }
                self.analytics_cache_timestamp = Some(Instant::now());
                drained += 1;
            }
            if drained >= ANALYTICS_MAX_UPDATES_PER_TICK {
                tracing::warn!(
                    "Analytics updates capped at {}",
                    ANALYTICS_MAX_UPDATES_PER_TICK
                );
            }
        }
    }
}
