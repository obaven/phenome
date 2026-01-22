//! Anomaly domain models.

use serde::{Deserialize, Serialize};

use crate::{ClusterId, MetricType, TimeRange};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    Critical,
    Warning,
    Info,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Anomaly {
    pub id: String,
    pub cluster_id: ClusterId,
    pub resource_id: String,
    pub detected_at: i64,
    pub metric_type: MetricType,
    pub severity: Severity,
    pub confidence: f64,
    pub description: String,
    pub baseline_value: f64,
    pub observed_value: f64,
    pub deviation_sigma: f64,
    #[serde(default)]
    pub related_metrics: Vec<String>,
    pub root_cause: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RootCauseAnalysis {
    pub summary: String,
    pub confidence: f64,
    #[serde(default)]
    pub related_metrics: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AnomalyFilter {
    pub cluster_id: Option<ClusterId>,
    pub resource_id: Option<String>,
    pub metric_type: Option<MetricType>,
    pub severity: Option<Severity>,
    pub time_range: Option<TimeRange>,
    pub limit: Option<u32>,
}
