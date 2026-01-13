use anyhow::Result;
use std::time::Duration;
use tokio::time::sleep;

use rotappo_domain::{MetricSample, MetricsQuery};

use crate::cluster_manager::ClusterManager;

#[derive(Debug, Clone)]
pub struct MetricsCollector {
    cluster_manager: ClusterManager,
    interval: Duration,
}

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
        loop {
            let _ = self.collect_once().await?;
            sleep(self.interval).await;
        }
    }
}

use async_trait::async_trait;
use rotappo_ports::MetricsPort;

#[async_trait]
impl MetricsPort for MetricsCollector {
    async fn collect_metrics(
        &self,
        cluster_id: rotappo_domain::ClusterId,
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
