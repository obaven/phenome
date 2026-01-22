use anyhow::Result;
use async_trait::async_trait;
use std::sync::{Arc, RwLock};
use std::time::Duration;

use phenome_domain::{
    AggregatedMetric, AggregatedQuery, Anomaly, AnomalyFilter, MetricSample, MetricType,
    MetricsQuery, Recommendation, RecommendationFilter, TimeRange, TimeSeries, TimeSeriesPoint,
};
use phenome_ports::AnalyticsPort;

use crate::aggregator::Aggregator;
use crate::grpc::MlClient;
use crate::storage::StoragePort;

#[derive(Clone)]
pub struct AnalyticsService {
    storage: Arc<dyn StoragePort>,
    aggregator: Aggregator,
    anomalies: Arc<RwLock<Vec<Anomaly>>>,
    recommendations: Arc<RwLock<Vec<Recommendation>>>,
    ml_client: MlClient,
}

impl std::fmt::Debug for AnalyticsService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let anomalies_count = self
            .anomalies
            .read()
            .map(|guard| guard.len())
            .unwrap_or(0);
        let recommendations_count = self
            .recommendations
            .read()
            .map(|guard| guard.len())
            .unwrap_or(0);
        f.debug_struct("AnalyticsService")
            .field("storage", &"StoragePort")
            .field("aggregator", &self.aggregator)
            .field("anomalies_count", &anomalies_count)
            .field("recommendations_count", &recommendations_count)
            .field("ml_client", &self.ml_client)
            .finish()
    }
}

impl AnalyticsService {
    pub fn new(storage: Arc<dyn StoragePort>, ml_client: MlClient) -> Self {
        Self {
            storage,
            aggregator: Aggregator::new(),
            anomalies: Arc::new(RwLock::new(Vec::new())),
            recommendations: Arc::new(RwLock::new(Vec::new())),
            ml_client,
        }
    }

    pub fn add_anomalies(&self, anomalies: Vec<Anomaly>) {
        if let Ok(mut store) = self.anomalies.write() {
            store.extend(anomalies);
        } else {
            tracing::error!("anomalies lock poisoned");
        }
    }

    pub fn add_recommendations(&self, recommendations: Vec<Recommendation>) {
        if let Ok(mut store) = self.recommendations.write() {
            store.extend(recommendations);
        } else {
            tracing::error!("recommendations lock poisoned");
        }
    }
}

#[async_trait]
impl AnalyticsPort for AnalyticsService {
    async fn record_metrics(&self, samples: Vec<MetricSample>) -> Result<()> {
        self.storage.insert_metrics(samples.clone()).await?;
        let aggregates = self
            .aggregator
            .aggregate_window(&samples, Duration::from_secs(3600))?;
        self.storage.insert_aggregated(aggregates).await?;
        Ok(())
    }

    async fn query_aggregated(&self, query: AggregatedQuery) -> Result<Vec<AggregatedMetric>> {
        self.storage.query_aggregated(query).await
    }

    async fn get_time_series(
        &self,
        resource_id: String,
        metric_type: MetricType,
        range: TimeRange,
    ) -> Result<TimeSeries> {
        let samples = self
            .storage
            .query_metrics(MetricsQuery {
                cluster_id: None,
                resource_type: None,
                resource_ids: vec![resource_id.clone()],
                metric_types: vec![metric_type],
                time_range: Some(range),
            })
            .await?;

        let mut points: Vec<TimeSeriesPoint> = samples
            .iter()
            .map(|sample| TimeSeriesPoint {
                timestamp: sample.timestamp,
                value: sample.value,
            })
            .collect();
        points.sort_by_key(|point| point.timestamp);

        let (cluster_id, unit) = samples
            .first()
            .map(|sample| (sample.cluster_id.clone(), sample.unit.clone()))
            .unwrap_or_default();

        Ok(TimeSeries {
            cluster_id,
            resource_id,
            metric_type,
            unit,
            points,
        })
    }

    async fn get_anomalies(&self, filter: AnomalyFilter) -> Result<Vec<Anomaly>> {
        // Trigger detection if filter is specific enough
        if let (Some(resource_id), Some(metric_type), Some(range)) =
            (&filter.resource_id, filter.metric_type, filter.time_range)
        {
            if let Ok(series) = self
                .get_time_series(resource_id.clone(), metric_type, range)
                .await
            {
                if let Ok(detected) = self.ml_client.detect_anomalies(&series).await {
                    // Persist anomalies
                    if let Err(e) = self.storage.insert_anomalies(detected.clone()).await {
                        tracing::error!("Failed to persist anomalies: {}", e);
                    }
                    self.add_anomalies(detected);
                }
            }
        }

        let store = self
            .anomalies
            .read()
            .map_err(|_| anyhow::anyhow!("anomalies lock poisoned"))?;
        let mut filtered: Vec<Anomaly> = store
            .iter()
            .filter(|anomaly| {
                filter
                    .cluster_id
                    .as_ref()
                    .map_or(true, |id| id == &anomaly.cluster_id)
                    && filter
                        .resource_id
                        .as_ref()
                        .map_or(true, |resource_id| resource_id == &anomaly.resource_id)
                    && filter
                        .metric_type
                        .as_ref()
                        .map_or(true, |metric_type| metric_type == &anomaly.metric_type)
                    && filter
                        .severity
                        .as_ref()
                        .map_or(true, |severity| severity == &anomaly.severity)
                    && filter.time_range.as_ref().map_or(true, |range| {
                        anomaly.detected_at >= range.start_ms && anomaly.detected_at <= range.end_ms
                    })
            })
            .cloned()
            .collect();

        if let Some(limit) = filter.limit {
            filtered.truncate(limit as usize);
        }

        Ok(filtered)
    }

    async fn get_recommendations(
        &self,
        filter: RecommendationFilter,
    ) -> Result<Vec<Recommendation>> {
        let store = self
            .recommendations
            .read()
            .map_err(|_| anyhow::anyhow!("recommendations lock poisoned"))?;
        let mut filtered: Vec<Recommendation> = store
            .iter()
            .filter(|rec| {
                filter
                    .cluster_id
                    .as_ref()
                    .map_or(true, |id| id == &rec.cluster_id)
                    && filter
                        .priority
                        .as_ref()
                        .map_or(true, |priority| priority == &rec.priority)
                    && filter
                        .status
                        .as_ref()
                        .map_or(true, |status| rec.status.kind() == *status)
            })
            .cloned()
            .collect();

        if let Some(limit) = filter.limit {
            filtered.truncate(limit as usize);
        }

        Ok(filtered)
    }

    async fn query_metrics(&self, query: MetricsQuery) -> Result<Vec<MetricSample>> {
        self.storage.query_metrics(query).await
    }
}
