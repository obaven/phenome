use super::anomaly::Severity;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    pub id: String,
    pub title: String,
    pub message: String,
    pub severity: Severity,
    pub timestamp: i64,
    pub read: bool,
    pub link: Option<String>,
    pub cluster_id: Option<String>,
    pub resource_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationChannel {
    pub id: String,
    pub name: String,
    pub enabled: bool,
    // Using simple String for now or a structured config
    pub config_json: String,
    // Or typed config if handled elsewhere
    #[serde(skip)]
    pub config: serde_json::Value,
}

impl Default for Notification {
    fn default() -> Self {
        Self {
            id: String::new(),
            title: String::new(),
            message: String::new(),
            severity: Severity::Info,
            timestamp: 0,
            read: false,
            link: None,
            cluster_id: None,
            resource_id: None,
        }
    }
}

impl Default for NotificationChannel {
    fn default() -> Self {
        Self {
            id: String::new(),
            name: String::new(),
            enabled: false,
            config_json: "{}".to_string(),
            config: serde_json::Value::Null,
        }
    }
}
