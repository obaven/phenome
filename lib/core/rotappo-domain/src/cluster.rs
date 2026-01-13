//! Cluster domain models.

use serde::{Deserialize, Serialize};

pub type ClusterId = String;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClusterHealth {
    Healthy,
    Degraded,
    Unreachable,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterMetadata {
    pub id: ClusterId,
    pub name: String,
    pub context: String,
    pub api_server: String,
    pub health_status: ClusterHealth,
    pub last_seen: i64,
    pub pod_count: u32,
    pub node_count: u32,
    pub namespace_count: u32,
}
