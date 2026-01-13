//! Analytics domain models.

use serde::{Deserialize, Serialize};
use std::time::Duration;

use crate::{ClusterId, MetricType, ResourceType};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct TimeRange {
    pub start_ms: i64,
    pub end_ms: i64,
}

impl TimeRange {
    pub fn duration_ms(&self) -> i64 {
        self.end_ms.saturating_sub(self.start_ms)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeSeriesPoint {
    pub timestamp: i64,
    pub value: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TimeSeries {
    pub cluster_id: ClusterId,
    pub resource_id: String,
    pub metric_type: MetricType,
    pub unit: String,
    #[serde(default)]
    pub points: Vec<TimeSeriesPoint>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeSeriesData {
    pub cluster_id: ClusterId,
    pub range: TimeRange,
    #[serde(default)]
    pub series: Vec<TimeSeries>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregatedMetric {
    pub cluster_id: ClusterId,
    pub resource_type: ResourceType,
    pub metric_type: MetricType,
    pub window_start: i64,
    pub window_duration: Duration,
    pub count: u64,
    pub sum: f64,
    pub min: f64,
    pub max: f64,
    pub avg: f64,
    pub p50: f64,
    pub p95: f64,
    pub p99: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MetricsQuery {
    pub cluster_id: Option<ClusterId>,
    pub resource_type: Option<ResourceType>,
    #[serde(default)]
    pub resource_ids: Vec<String>,
    #[serde(default)]
    pub metric_types: Vec<MetricType>,
    pub time_range: Option<TimeRange>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregatedQuery {
    pub cluster_id: Option<ClusterId>,
    pub resource_type: Option<ResourceType>,
    #[serde(default)]
    pub metric_types: Vec<MetricType>,
    pub window_duration: Duration,
    pub time_range: Option<TimeRange>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScalingPrediction {
    pub resource_id: String,
    pub generated_at: i64,
    pub horizon: Duration,
    pub predicted_value: f64,
    pub unit: String,
}
