use anyhow::Result;
use polars::prelude::*;
use rotappo_domain::MetricSample;

pub struct AnalyticsEngine;

impl AnalyticsEngine {
    pub fn samples_to_df(samples: Vec<MetricSample>) -> Result<DataFrame> {
        if samples.is_empty() {
            return Ok(DataFrame::default());
        }

        let cluster_ids: Vec<String> = samples.iter().map(|s| s.cluster_id.clone()).collect();
        // ResourceType/MetricType are enums, convert to string for Polars
        let resource_types: Vec<String> = samples
            .iter()
            .map(|s| serde_json::to_string(&s.resource_type).unwrap_or_default())
            .collect();
        let resource_ids: Vec<String> = samples.iter().map(|s| s.resource_id.clone()).collect();
        let metric_types: Vec<String> = samples
            .iter()
            .map(|s| serde_json::to_string(&s.metric_type).unwrap_or_default())
            .collect();
        let timestamps: Vec<i64> = samples.iter().map(|s| s.timestamp).collect();
        let values: Vec<f64> = samples.iter().map(|s| s.value).collect();
        let units: Vec<String> = samples.iter().map(|s| s.unit.clone()).collect();

        let df = df!(
            "cluster_id" => cluster_ids,
            "resource_type" => resource_types,
            "resource_id" => resource_ids,
            "metric_type" => metric_types,
            "timestamp" => timestamps,
            "value" => values,
            "unit" => units
        )?;

        Ok(df)
    }

    pub fn aggregate_by_window(df: &DataFrame, window_ms: i64) -> Result<DataFrame> {
        // Example Polars aggregation
        // This is where "Analytics-Enhanced" power comes in.
        // But for now just simple conversion is enough for "Integration".
        Ok(DataFrame::default()) // Stub for now
    }
}
