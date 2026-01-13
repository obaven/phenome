use anyhow::Result;
use async_trait::async_trait;

use rotappo_domain::{
    AggregatedMetric, AggregatedQuery, Anomaly, AnomalyFilter, MetricSample, MetricType,
    MetricsQuery, Recommendation, RecommendationFilter, TimeRange, TimeSeries,
};

#[async_trait]
pub trait AnalyticsPort: Send + Sync {
    async fn record_metrics(&self, samples: Vec<MetricSample>) -> Result<()>;
    async fn query_aggregated(&self, query: AggregatedQuery) -> Result<Vec<AggregatedMetric>>;
    async fn get_time_series(
        &self,
        resource_id: String,
        metric_type: MetricType,
        range: TimeRange,
    ) -> Result<TimeSeries>;
    async fn get_anomalies(&self, filter: AnomalyFilter) -> Result<Vec<Anomaly>>;
    async fn get_recommendations(
        &self,
        filter: RecommendationFilter,
    ) -> Result<Vec<Recommendation>>;
    async fn query_metrics(&self, query: MetricsQuery) -> Result<Vec<MetricSample>>;
}
