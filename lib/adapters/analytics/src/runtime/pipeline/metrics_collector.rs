use anyhow::Result;
use std::time::Duration;
use tokio::sync::watch;
use tokio::time::{interval, timeout};

use phenome_domain::{MetricSample, MetricsQuery};

use crate::cluster_manager::ClusterManager;

#[derive(Debug, Clone)]
pub struct MetricsCollector {
    cluster_manager: ClusterManager,
    interval: Duration,
}

const MAX_COLLECTION_DURATION: Duration = Duration::from_secs(30);

impl MetricsCollector {
    pub fn new(cluster_manager: ClusterManager, interval: Duration) -> Self {
        Self {
            cluster_manager,
            interval,
        }
    }

    pub async fn collect_once(&self) -> Result<Vec<MetricSample>> {
        let query = MetricsQuery::default();
        let results = self.cluster_manager.query_all_clusters(query).await;
        Ok(results
            .into_iter()
            .flat_map(|(_, result)| result.unwrap_or_default())
            .collect())
    }

    pub async fn run_polling_loop(&self) -> Result<()> {
        let (_tx, rx) = watch::channel(false);
        self.run_polling_loop_with_shutdown(rx).await
    }

    pub async fn run_polling_loop_with_shutdown(
        &self,
        mut shutdown: watch::Receiver<bool>,
    ) -> Result<()> {
        let mut tick = interval(self.interval);
        loop {
            tokio::select! {
                result = shutdown.changed() => {
                    if result.is_err() || *shutdown.borrow() {
                        break;
                    }
                }
                _ = tick.tick() => {
                    match timeout(MAX_COLLECTION_DURATION, self.collect_once()).await {
                        Ok(Ok(_)) => {}
                        Ok(Err(err)) => {
                            tracing::error!("Metrics poll failed: {}", err);
                        }
                        Err(_) => {
                            tracing::warn!(
                                "Metrics poll exceeded {:?} budget",
                                MAX_COLLECTION_DURATION
                            );
                        }
                    }
                }
            }
        }
        Ok(())
    }
}

use async_trait::async_trait;
use phenome_ports::MetricsPort;

#[async_trait]
impl MetricsPort for MetricsCollector {
    async fn collect_metrics(
        &self,
        cluster_id: phenome_domain::ClusterId,
    ) -> Result<Vec<MetricSample>> {
        let query = MetricsQuery::default();
        self.cluster_manager.query_metrics(&cluster_id, query).await
    }

    async fn query_metrics(&self, query: MetricsQuery) -> Result<Vec<MetricSample>> {
        let cluster_id = query.cluster_id.clone();
        if let Some(cid) = cluster_id {
            self.cluster_manager.query_metrics(&cid, query).await
        } else {
            let results = self.cluster_manager.query_all_clusters(query).await;
            Ok(results
                .into_iter()
                .flat_map(|(_, result)| result.unwrap_or_default())
                .collect())
        }
    }
}
