//! Domain health status types for adapters and presentation.

use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ComponentHealthStatus {
    Healthy,
    Degraded(String),
    Unhealthy(String),
}

#[derive(Debug, Clone, Default)]
pub struct HealthSnapshot {
    pub health: HashMap<String, ComponentHealthStatus>,
    pub last_error: Option<String>,
    pub cache_ready: bool,
}
