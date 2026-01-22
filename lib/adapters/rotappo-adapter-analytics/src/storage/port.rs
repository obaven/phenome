use anyhow::Result;
use async_trait::async_trait;

use phenome_domain::{AggregatedMetric, AggregatedQuery, MetricSample, MetricsQuery};

#[async_trait]
pub trait StoragePort: Send + Sync {
    async fn insert_metrics(&self, samples: Vec<MetricSample>) -> Result<()>;
    async fn query_metrics(&self, query: MetricsQuery) -> Result<Vec<MetricSample>>;
    async fn insert_aggregated(&self, metrics: Vec<AggregatedMetric>) -> Result<()>;
    async fn query_aggregated(&self, query: AggregatedQuery) -> Result<Vec<AggregatedMetric>>;
    async fn insert_anomalies(&self, anomalies: Vec<phenome_domain::Anomaly>) -> Result<()>;
    async fn cleanup_retention(&self) -> Result<()>;

    // Scheduler methods
    async fn insert_schedule(&self, action: phenome_domain::ScheduledAction) -> Result<()>;
    async fn update_schedule(&self, action: phenome_domain::ScheduledAction) -> Result<()>;
    async fn get_all_schedules(&self) -> Result<Vec<phenome_domain::ScheduledAction>>;
}
