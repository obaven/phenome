use anyhow::{Context, Result};

use phenome_adapter_analytics::grpc::analytics::QueryMetricsRequest;
use phenome_domain::MetricSample;

use super::AnalyticsClient;

pub(super) async fn fetch_metrics(client: &AnalyticsClient) -> Result<Vec<MetricSample>> {
    let mut grpc = client.client.clone();
    let request = QueryMetricsRequest {
        cluster_id: None,
        resource_type: None,
        resource_ids: Vec::new(),
        metric_types: Vec::new(),
        time_range: None,
    };
    let response = grpc.query_metrics(request).await?;
    let samples = response.into_inner().samples;

    samples
        .into_iter()
        .map(|s| s.try_into())
        .collect::<Result<Vec<_>, _>>()
        .context("failed to convert metrics")
}
