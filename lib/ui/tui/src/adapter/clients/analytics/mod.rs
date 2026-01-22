use anyhow::Result;
use tonic::transport::Channel;

use phenome_adapter_analytics::grpc::analytics::analytics_service_client::AnalyticsServiceClient;
use phenome_domain::{Anomaly, MetricSample, Recommendation};

mod anomalies;
mod connection;
mod metrics;
mod recommendations;

#[derive(Debug, Clone)]
pub struct AnalyticsClient {
    client: AnalyticsServiceClient<Channel>,
}

impl AnalyticsClient {
    pub async fn connect_from_env() -> Result<Self> {
        connection::connect_from_env().await
    }

    pub async fn fetch_metrics(&self) -> Result<Vec<MetricSample>> {
        metrics::fetch_metrics(self).await
    }

    pub async fn fetch_anomalies(&self) -> Result<Vec<Anomaly>> {
        anomalies::fetch_anomalies(self).await
    }

    pub async fn fetch_recommendations(&self) -> Result<Vec<Recommendation>> {
        recommendations::fetch_recommendations(self).await
    }
}
