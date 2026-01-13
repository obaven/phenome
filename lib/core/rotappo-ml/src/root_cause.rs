use anyhow::Result;

use rotappo_domain::{Anomaly, RootCauseAnalysis};

#[derive(Debug, Clone, Default)]
pub struct RootCauseEngine;

impl RootCauseEngine {
    pub fn new() -> Self {
        Self
    }

    pub fn analyze(&self, anomaly: &Anomaly) -> Result<RootCauseAnalysis> {
        Ok(RootCauseAnalysis {
            summary: format!("No root cause available for anomaly {}", anomaly.id),
            confidence: 0.0,
            related_metrics: Vec::new(),
        })
    }
}
