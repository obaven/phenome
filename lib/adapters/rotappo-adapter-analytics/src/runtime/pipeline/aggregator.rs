use anyhow::Result;
use phenome_domain::{AggregatedMetric, MetricSample, MetricType, ResourceType};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::watch;
use tokio::time::{interval, timeout};

use crate::storage::StoragePort;

#[derive(Debug, Clone, Default)]
pub struct Aggregator;

const RETENTION_INTERVAL: Duration = Duration::from_secs(60 * 60);
const RETENTION_BUDGET: Duration = Duration::from_secs(30);

impl Aggregator {
    pub fn new() -> Self {
        Self
    }

    pub async fn run_hourly(storage: Arc<dyn StoragePort>) {
        let (_tx, rx) = watch::channel(false);
        Self::run_hourly_with_shutdown(storage, rx).await;
    }

    pub async fn run_hourly_with_shutdown(
        storage: Arc<dyn StoragePort>,
        mut shutdown: watch::Receiver<bool>,
    ) {
        let mut tick = interval(RETENTION_INTERVAL);
        loop {
            tokio::select! {
                result = shutdown.changed() => {
                    if result.is_err() || *shutdown.borrow() {
                        break;
                    }
                }
                _ = tick.tick() => {
                    match timeout(RETENTION_BUDGET, storage.cleanup_retention()).await {
                        Ok(Ok(())) => {}
                        Ok(Err(err)) => {
                            tracing::error!("Retention cleanup failed: {}", err);
                        }
                        Err(_) => {
                            tracing::warn!(
                                "Retention cleanup exceeded {:?} budget",
                                RETENTION_BUDGET
                            );
                        }
                    }
                }
            }
        }
    }

    pub fn aggregate_window(
        &self,
        samples: &[MetricSample],
        window_duration: Duration,
    ) -> Result<Vec<AggregatedMetric>> {
        let window_secs = window_duration.as_secs() as i64;
        if window_secs == 0 {
            return Ok(Vec::new());
        }

        // Key: (ClusterId, ResourceType(json), MetricType(json), window_start)
        // Using Strings for JSON-encoded enums to implement Hash
        type GroupKey = (String, String, String, i64);
        let mut groups: HashMap<GroupKey, Vec<f64>> = HashMap::new();

        for s in samples {
            // Timestamp in ms. Convert to seconds, bucketize, convert back to ms.
            let ts_sec = s.timestamp / 1000;
            let window_start_sec = (ts_sec / window_secs) * window_secs;
            let window_start = window_start_sec * 1000;

            let key = (
                s.cluster_id.clone(),
                serde_json::to_string(&s.resource_type)?,
                serde_json::to_string(&s.metric_type)?,
                window_start,
            );
            groups.entry(key).or_default().push(s.value);
        }

        let mut results = Vec::new();
        for ((cluster_id, r_type_str, m_type_str, window_start), values) in groups {
            let count = values.len() as u64;
            let sum: f64 = values.iter().sum();

            let min = values.iter().cloned().fold(f64::INFINITY, f64::min);
            let max = values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
            let avg = if count > 0 { sum / count as f64 } else { 0.0 };

            let mut sorted = values.clone();
            sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

            let p50 = percentile(&sorted, 0.5);
            let p95 = percentile(&sorted, 0.95);
            let p99 = percentile(&sorted, 0.99);

            let resource_type: ResourceType = serde_json::from_str(&r_type_str)?;
            let metric_type: MetricType = serde_json::from_str(&m_type_str)?;

            results.push(AggregatedMetric {
                cluster_id,
                resource_type,
                metric_type,
                window_start,
                window_duration,
                count,
                sum,
                min,
                max,
                avg,
                p50,
                p95,
                p99,
            });
        }

        Ok(results)
    }
}

fn percentile(sorted: &[f64], pct: f64) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    let idx = (sorted.len() as f64 * pct).ceil() as usize;
    if idx == 0 {
        return sorted[0];
    }
    if idx >= sorted.len() {
        return sorted[sorted.len() - 1];
    }
    sorted[idx - 1]
}
