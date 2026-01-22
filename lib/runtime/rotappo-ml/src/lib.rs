//! Machine learning models for rotappo analytics.

mod detection;
mod recommendations;
mod scaling;

pub use detection::anomaly_detection::AnomalyDetector;
pub use detection::root_cause::RootCauseEngine;
pub use recommendations::recommendations::RecommendationEngine;
pub use scaling::scaling_prediction::ScalingPredictor;
