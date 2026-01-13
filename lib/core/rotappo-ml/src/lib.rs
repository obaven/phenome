//! Machine learning models for rotappo analytics.

pub mod anomaly_detection;
pub mod recommendations;
pub mod root_cause;
pub mod scaling_prediction;

pub use anomaly_detection::AnomalyDetector;
pub use recommendations::RecommendationEngine;
pub use root_cause::RootCauseEngine;
pub use scaling_prediction::ScalingPredictor;

#[cfg(test)]
mod anomaly_detection_test;
#[cfg(test)]
mod scaling_prediction_test;
