use anyhow::Result;
use async_trait::async_trait;

use rotappo_domain::{ClusterId, MetricSample, MetricsQuery};

#[async_trait]
pub trait MetricsPort: Send + Sync {
    async fn collect_metrics(&self, cluster_id: ClusterId) -> Result<Vec<MetricSample>>;
    async fn query_metrics(&self, query: MetricsQuery) -> Result<Vec<MetricSample>>;
}
