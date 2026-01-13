use anyhow::Result;
use async_trait::async_trait;

use rotappo_domain::{AggregatedMetric, AggregatedQuery, MetricSample, MetricsQuery};

use super::port::StoragePort;

#[derive(Debug, Clone)]
pub struct PostgresStorage;

impl PostgresStorage {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl StoragePort for PostgresStorage {
    async fn insert_metrics(&self, _samples: Vec<MetricSample>) -> Result<()> {
        anyhow::bail!("postgres storage not implemented")
    }

    async fn query_metrics(&self, _query: MetricsQuery) -> Result<Vec<MetricSample>> {
        anyhow::bail!("postgres storage not implemented")
    }

    async fn insert_aggregated(&self, _metrics: Vec<AggregatedMetric>) -> Result<()> {
        anyhow::bail!("postgres storage not implemented")
    }

    async fn query_aggregated(&self, _query: AggregatedQuery) -> Result<Vec<AggregatedMetric>> {
        anyhow::bail!("postgres storage not implemented")
    }
}
