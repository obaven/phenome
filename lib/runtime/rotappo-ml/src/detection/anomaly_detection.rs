use anyhow::Result;

use phenome_domain::{Anomaly, Severity, TimeSeriesData};

#[derive(Debug, Clone)]
pub struct AnomalyDetector {
    sigma_threshold: f64,
    min_confidence: f64,
}

impl Default for AnomalyDetector {
    fn default() -> Self {
        Self {
            sigma_threshold: 3.0,
            min_confidence: 0.7,
        }
    }
}

impl AnomalyDetector {
    pub fn detect(&self, data: &TimeSeriesData) -> Result<Vec<Anomaly>> {
        let mut anomalies = Vec::new();
        for series in &data.series {
            let values: Vec<f64> = series
                .points
                .iter()
                .map(|point| point.value)
                .filter(|value| value.is_finite())
                .collect();
            if values.len() < 10 {
                // Not enough data for ML, use Z-score or skip
                continue;
            }

            // Fallback Z-score for small data or if ML fails/is overkill
            let mean = values.iter().sum::<f64>() / values.len() as f64;
            let variance =
                values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / values.len() as f64;
            let stddev = variance.sqrt();
            let latest = if let Some(l) = series.points.last() {
                l
            } else {
                continue;
            };

            let mut detected_anomaly = None;

            // Isolation Forest integration is deferred; current detector uses Z-score only.

            // Fallback or confirm with Z-score
            if detected_anomaly.is_none() && stddev > f64::EPSILON {
                let deviation = (latest.value - mean).abs() / stddev;
                if deviation >= self.sigma_threshold {
                    // Z-score anomaly
                    let confidence = (deviation / (self.sigma_threshold * 1.5)).min(0.99);
                    if confidence >= self.min_confidence {
                        let severity = if confidence > 0.9 {
                            Severity::Critical
                        } else {
                            Severity::Warning
                        };
                        detected_anomaly = Some((
                            severity,
                            confidence,
                            format!("{:.2} sigma deviation", deviation),
                        ));
                    }
                }
            }

            if let Some((severity, confidence, desc)) = detected_anomaly {
                anomalies.push(Anomaly {
                    id: format!("{}-{}", series.resource_id, latest.timestamp),
                    cluster_id: data.cluster_id.clone(),
                    resource_id: series.resource_id.clone(),
                    detected_at: latest.timestamp,
                    metric_type: series.metric_type,
                    severity,
                    confidence,
                    description: desc,
                    baseline_value: mean,
                    observed_value: latest.value,
                    deviation_sigma: (latest.value - mean).abs() / stddev.max(f64::EPSILON),
                    related_metrics: Vec::new(),
                    root_cause: None,
                });
            }
        }

        Ok(anomalies)
    }
}
