use anyhow::Result;
use chrono::Utc;

use rotappo_domain::{
    Priority, Recommendation, RecommendationAction, RecommendationStatus, RecommendationType,
};

#[derive(Debug, Clone, Default)]
pub struct RecommendationEngine;

impl RecommendationEngine {
    pub fn new() -> Self {
        Self
    }

    pub fn generate(&self, cluster_id: String) -> Result<Vec<Recommendation>> {
        Ok(vec![Recommendation {
            id: format!("rec-{}", Utc::now().timestamp_millis()),
            cluster_id,
            created_at: Utc::now().timestamp_millis(),
            recommendation_type: RecommendationType::OptimizeResources,
            priority: Priority::Low,
            confidence: 0.0,
            title: "No recommendations available".to_string(),
            description: "No actionable insights found for this cluster.".to_string(),
            impact_estimate: "No impact".to_string(),
            cost_impact: None,
            action: RecommendationAction::ScaleDeployment {
                name: "unknown".to_string(),
                from: 0,
                to: 0,
            },
            status: RecommendationStatus::Pending,
        }])
    }
}
