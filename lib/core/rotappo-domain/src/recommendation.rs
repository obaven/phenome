//! Recommendation domain models.

use serde::{Deserialize, Serialize};

use crate::ClusterId;

pub type ScheduleId = String;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecommendationType {
    ScaleUp,
    ScaleDown,
    OptimizeResources,
    AdjustLimits,
    StorageOptimization,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Priority {
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostImpact {
    pub daily_change: f64,
    pub currency: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    pub cpu: Option<String>,
    pub memory: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum RecommendationAction {
    ScaleDeployment {
        name: String,
        from: u32,
        to: u32,
    },
    UpdateResourceLimits {
        resource: String,
        limits: ResourceLimits,
    },
    ReclaimStorage {
        volume: String,
        size_gb: u64,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum RecommendationStatus {
    Pending,
    Scheduled { execute_at: i64 },
    Applied { applied_at: i64 },
    Dismissed { reason: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecommendationStatusKind {
    Pending,
    Scheduled,
    Applied,
    Dismissed,
}

impl RecommendationStatus {
    pub fn kind(&self) -> RecommendationStatusKind {
        match self {
            RecommendationStatus::Pending => RecommendationStatusKind::Pending,
            RecommendationStatus::Scheduled { .. } => RecommendationStatusKind::Scheduled,
            RecommendationStatus::Applied { .. } => RecommendationStatusKind::Applied,
            RecommendationStatus::Dismissed { .. } => RecommendationStatusKind::Dismissed,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recommendation {
    pub id: String,
    pub cluster_id: ClusterId,
    pub created_at: i64,
    pub recommendation_type: RecommendationType,
    pub priority: Priority,
    pub confidence: f64,
    pub title: String,
    pub description: String,
    pub impact_estimate: String,
    pub cost_impact: Option<CostImpact>,
    pub action: RecommendationAction,
    pub status: RecommendationStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RecommendationFilter {
    pub cluster_id: Option<ClusterId>,
    pub priority: Option<Priority>,
    pub status: Option<RecommendationStatusKind>,
    pub limit: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledAction {
    pub id: ScheduleId,
    pub execute_at: i64,
    pub recommendation_id: String,
    pub action: RecommendationAction,
    pub status: ScheduleStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ScheduleStatus {
    Pending,
    Executing,
    Completed,
    Failed { error: String },
    Cancelled,
}
