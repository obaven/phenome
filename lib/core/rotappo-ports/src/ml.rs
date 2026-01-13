use anyhow::Result;
use async_trait::async_trait;
use std::time::Duration;

use rotappo_domain::{
    Anomaly, ClusterId, Recommendation, ScalingPrediction, TimeSeriesData,
};

#[async_trait]
pub trait MLPort: Send + Sync {
    async fn detect_anomalies(&self, data: TimeSeriesData) -> Result<Vec<Anomaly>>;
    async fn predict_scaling_needs(
        &self,
        resource_id: String,
        horizon: Duration,
    ) -> Result<ScalingPrediction>;
    async fn generate_recommendations(
        &self,
        cluster_id: ClusterId,
    ) -> Result<Vec<Recommendation>>;
}
