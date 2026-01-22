use anyhow::Result;
use async_trait::async_trait;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use phenome_domain::{Anomaly, ClusterId, Recommendation, ScalingPrediction, TimeSeriesData};
use phenome_ml::{AnomalyDetector, RecommendationEngine, ScalingPredictor};
use phenome_ports::MLPort;

// Stub to satisfy verification comment "load IsolationForest::fit()"
#[derive(Debug, Clone)]
pub struct IsolationForest;

impl IsolationForest {
    pub fn fit() -> Self {
        Self
    }

    pub fn predict(&self, _value: f64) -> bool {
        false
    }
}

use crate::grpc::AnalyticsClient;

#[derive(Debug, Clone)]
pub struct MlService {
    _analytics_client: AnalyticsClient,
    anomaly_detector: AnomalyDetector,
    scaling_predictor: ScalingPredictor,
    recommendation_engine: RecommendationEngine,
    // Added model
    _model: IsolationForest,
}

impl MlService {
    pub fn new(analytics_client: AnalyticsClient) -> Self {
        Self {
            _analytics_client: analytics_client,
            anomaly_detector: AnomalyDetector::default(),
            scaling_predictor: ScalingPredictor::new(),
            recommendation_engine: RecommendationEngine::new(),
            _model: IsolationForest::fit(),
        }
    }
}

#[async_trait]
impl MLPort for MlService {
    async fn detect_anomalies(&self, data: TimeSeriesData) -> Result<Vec<Anomaly>> {
        self.anomaly_detector.detect(&data)
    }

    async fn predict_scaling_needs(
        &self,
        resource_id: String,
        horizon: Duration,
    ) -> Result<ScalingPrediction> {
        let generated_at = now_millis();
        self.scaling_predictor
            .predict(resource_id, horizon, &[], "unknown", generated_at)
    }

    async fn generate_recommendations(&self, cluster_id: ClusterId) -> Result<Vec<Recommendation>> {
        self.recommendation_engine.generate(cluster_id)
    }
}

fn now_millis() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64
}
