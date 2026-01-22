use anyhow::Result;
use std::time::Duration;

use phenome_domain::ScalingPrediction;

#[derive(Debug, Clone, Default)]
pub struct ScalingPredictor;

impl ScalingPredictor {
    pub fn new() -> Self {
        Self
    }

    pub fn predict(
        &self,
        resource_id: String,
        horizon: Duration,
        history: &[f64],
        unit: &str,
        generated_at: i64,
    ) -> Result<ScalingPrediction> {
        let predicted_value = if history.is_empty() {
            0.0
        } else {
            history.iter().sum::<f64>() / history.len() as f64
        };

        Ok(ScalingPrediction {
            resource_id,
            generated_at,
            horizon,
            predicted_value,
            unit: unit.to_string(),
        })
    }
}
