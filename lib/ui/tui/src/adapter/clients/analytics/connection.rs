use anyhow::{Context, Result};

use phenome_adapter_analytics::grpc::analytics::analytics_service_client::AnalyticsServiceClient;

use super::AnalyticsClient;

pub(super) async fn connect_from_env() -> Result<AnalyticsClient> {
    let endpoint =
        std::env::var("ROTAPPO_ANALYTICS_URL").unwrap_or_else(|_| "http://localhost:50051".into());
    let client = AnalyticsServiceClient::connect(endpoint)
        .await
        .context("failed to connect to analytics service")?;
    Ok(AnalyticsClient { client })
}
