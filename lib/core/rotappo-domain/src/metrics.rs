//! Metrics domain models.

use serde::{Deserialize, Serialize};

use crate::ClusterId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ResourceType {
    #[default]
    Pod,
    Node,
    Container,
    Service,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum MetricType {
    #[default]
    CpuUsage,
    MemoryUsage,
    NetworkIn,
    NetworkOut,
    DiskRead,
    DiskWrite,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricSample {
    pub cluster_id: ClusterId,
    pub resource_type: ResourceType,
    pub resource_id: String,
    pub metric_type: MetricType,
    pub timestamp: i64,
    pub value: f64,
    pub unit: String,
}
