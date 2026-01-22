use anyhow::Result;

use rotappo_adapter_analytics::grpc::analytics::GetAnomaliesRequest;
use phenome_domain::{Anomaly, MetricType, Severity};

use super::AnalyticsClient;

pub(super) async fn fetch_anomalies(client: &AnalyticsClient) -> Result<Vec<Anomaly>> {
    let mut grpc = client.client.clone();
    let request = GetAnomaliesRequest {
        limit: Some(50),
        ..Default::default()
    };
    let response = grpc.get_anomalies(request).await?;
    let anomalies = response.into_inner().anomalies;

    Ok(anomalies
        .into_iter()
        .map(|a| {
            let metric_type = map_metric_type(a.metric_type());
            let severity = map_severity(a.severity());
            Anomaly {
                id: a.id,
                cluster_id: a.cluster_id,
                resource_id: a.resource_id,
                detected_at: a.detected_at,
                metric_type,
                severity,
                confidence: a.confidence,
                description: a.description,
                baseline_value: a.baseline_value,
                observed_value: a.observed_value,
                deviation_sigma: a.deviation_sigma,
                related_metrics: a.related_metrics,
                root_cause: a.root_cause,
            }
        })
        .collect())
}

fn map_metric_type(metric: rotappo_adapter_analytics::grpc::analytics::MetricType) -> MetricType {
    match metric {
        rotappo_adapter_analytics::grpc::analytics::MetricType::CpuUsage => MetricType::CpuUsage,
        rotappo_adapter_analytics::grpc::analytics::MetricType::MemoryUsage => {
            MetricType::MemoryUsage
        }
        rotappo_adapter_analytics::grpc::analytics::MetricType::NetworkIn => MetricType::NetworkIn,
        rotappo_adapter_analytics::grpc::analytics::MetricType::NetworkOut => {
            MetricType::NetworkOut
        }
        rotappo_adapter_analytics::grpc::analytics::MetricType::DiskRead => MetricType::DiskRead,
        rotappo_adapter_analytics::grpc::analytics::MetricType::DiskWrite => MetricType::DiskWrite,
        _ => MetricType::CpuUsage,
    }
}

fn map_severity(severity: rotappo_adapter_analytics::grpc::analytics::Severity) -> Severity {
    match severity {
        rotappo_adapter_analytics::grpc::analytics::Severity::Critical => Severity::Critical,
        rotappo_adapter_analytics::grpc::analytics::Severity::Warning => Severity::Warning,
        rotappo_adapter_analytics::grpc::analytics::Severity::Info => Severity::Info,
        _ => Severity::Info,
    }
}
