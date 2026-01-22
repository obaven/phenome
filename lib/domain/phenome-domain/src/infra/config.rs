//! Rotappo configuration schema and loader.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RotappoConfig {
    pub deployment: DeploymentConfig,
    pub analytics: AnalyticsConfig,
    pub ml: MlConfig,
    pub clusters: Vec<ClusterConfig>,
    pub services: ServicesConfig,
    pub notifications: NotificationsConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentConfig {
    pub mode: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsConfig {
    pub storage: String,
    pub sqlite_path: String,
    pub retention: RetentionConfig,
    pub collection: CollectionConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetentionConfig {
    pub full_resolution_days: i64,
    pub aggregated_days: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionConfig {
    pub interval_seconds: u64,
    pub batch_size: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MlConfig {
    pub models: MlModelsConfig,
    pub thresholds: MlThresholdsConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MlModelsConfig {
    pub anomaly_detection: String,
    pub scaling_prediction: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MlThresholdsConfig {
    pub critical_confidence: f64,
    pub warning_confidence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterConfig {
    pub name: String,
    pub context: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServicesConfig {
    pub analytics_url: String,
    pub ml_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationsConfig {
    pub channels: Vec<NotificationChannelConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum NotificationChannelConfig {
    Ntfy { url: String, topic: String },
}

impl RotappoConfig {
    pub fn load_from_path(path: &Path) -> anyhow::Result<Self> {
        let contents = fs::read_to_string(path)?;
        let config = serde_yaml::from_str(&contents)?;
        Ok(config)
    }
}
