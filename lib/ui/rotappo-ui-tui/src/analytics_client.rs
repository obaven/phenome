use anyhow::{Context, Result};
use tonic::transport::Channel;

use rotappo_adapter_analytics::grpc::analytics::analytics_service_client::AnalyticsServiceClient;
use rotappo_adapter_analytics::grpc::analytics::{
    GetAnomaliesRequest, GetRecommendationsRequest, QueryMetricsRequest,
};
use rotappo_domain::{Anomaly, MetricSample, Recommendation};

#[derive(Debug, Clone)]
pub struct AnalyticsClient {
    client: AnalyticsServiceClient<Channel>,
}

impl AnalyticsClient {
    pub async fn connect_from_env() -> Result<Self> {
        let endpoint = std::env::var("ROTAPPO_ANALYTICS_URL")
            .unwrap_or_else(|_| "http://localhost:50051".into());
        let client = AnalyticsServiceClient::connect(endpoint)
            .await
            .context("failed to connect to analytics service")?;
        Ok(Self { client })
    }

    pub async fn fetch_metrics(&self) -> Result<Vec<MetricSample>> {
        let mut client = self.client.clone();
        let request = QueryMetricsRequest {
            cluster_id: None,
            resource_type: None, // Fetch all types
            resource_ids: Vec::new(),
            metric_types: Vec::new(),
            time_range: None, // Last available or default
        };
        let response = client.query_metrics(request).await?;
        let samples = response.into_inner().samples;

        samples
            .into_iter()
            .map(|s| s.try_into())
            .collect::<Result<Vec<_>, _>>()
            .context("failed to convert metrics")
    }

    pub async fn fetch_anomalies(&self) -> Result<Vec<Anomaly>> {
        let mut client = self.client.clone();
        let request = GetAnomaliesRequest {
            limit: Some(50),
            ..Default::default()
        };
        let response = client.get_anomalies(request).await?;
        let anomalies = response.into_inner().anomalies;

        let domain_anomalies = anomalies
            .into_iter()
            .map(|a| {
                let metric_type = match a.metric_type() {
                    rotappo_adapter_analytics::grpc::analytics::MetricType::CpuUsage => {
                        rotappo_domain::MetricType::CpuUsage
                    }
                    rotappo_adapter_analytics::grpc::analytics::MetricType::MemoryUsage => {
                        rotappo_domain::MetricType::MemoryUsage
                    }
                    rotappo_adapter_analytics::grpc::analytics::MetricType::NetworkIn => {
                        rotappo_domain::MetricType::NetworkIn
                    }
                    rotappo_adapter_analytics::grpc::analytics::MetricType::NetworkOut => {
                        rotappo_domain::MetricType::NetworkOut
                    }
                    rotappo_adapter_analytics::grpc::analytics::MetricType::DiskRead => {
                        rotappo_domain::MetricType::DiskRead
                    }
                    rotappo_adapter_analytics::grpc::analytics::MetricType::DiskWrite => {
                        rotappo_domain::MetricType::DiskWrite
                    }
                    _ => rotappo_domain::MetricType::CpuUsage, // Fallback
                };
                let severity = match a.severity() {
                    rotappo_adapter_analytics::grpc::analytics::Severity::Critical => {
                        rotappo_domain::Severity::Critical
                    }
                    rotappo_adapter_analytics::grpc::analytics::Severity::Warning => {
                        rotappo_domain::Severity::Warning
                    }
                    rotappo_adapter_analytics::grpc::analytics::Severity::Info => {
                        rotappo_domain::Severity::Info
                    }
                    _ => rotappo_domain::Severity::Info,
                };

                rotappo_domain::Anomaly {
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
            .collect::<Vec<_>>();

        Ok(domain_anomalies)
    }

    pub async fn fetch_recommendations(&self) -> Result<Vec<Recommendation>> {
        let mut client = self.client.clone();
        let request = GetRecommendationsRequest {
            limit: Some(20),
            ..Default::default()
        };
        let response = client.get_recommendations(request).await?;
        let recs = response.into_inner().recommendations;

        let domain_recs = recs.into_iter()
            .map(|r| {
                let rec_type = match r.recommendation_type() {
                    rotappo_adapter_analytics::grpc::analytics::RecommendationType::ScaleUp => {
                        rotappo_domain::RecommendationType::ScaleUp
                    }
                    rotappo_adapter_analytics::grpc::analytics::RecommendationType::ScaleDown => {
                        rotappo_domain::RecommendationType::ScaleDown
                    }
                    rotappo_adapter_analytics::grpc::analytics::RecommendationType::OptimizeResources => {
                        rotappo_domain::RecommendationType::OptimizeResources
                    }
                    rotappo_adapter_analytics::grpc::analytics::RecommendationType::AdjustLimits => {
                        rotappo_domain::RecommendationType::AdjustLimits
                    }
                    rotappo_adapter_analytics::grpc::analytics::RecommendationType::StorageOptimizations => {
                        rotappo_domain::RecommendationType::StorageOptimization
                    }
                    _ => rotappo_domain::RecommendationType::OptimizeResources, // Default/Fallback
                };
                let priority = match r.priority() {
                    rotappo_adapter_analytics::grpc::analytics::Priority::High => {
                        rotappo_domain::Priority::High
                    }
                    rotappo_adapter_analytics::grpc::analytics::Priority::Medium => {
                        rotappo_domain::Priority::Medium
                    }
                    rotappo_adapter_analytics::grpc::analytics::Priority::Low => {
                        rotappo_domain::Priority::Low
                    }
                    _ => rotappo_domain::Priority::Medium,
                };

                rotappo_domain::Recommendation {
                    id: r.id,
                    cluster_id: r.cluster_id,
                    created_at: r.created_at,
                    recommendation_type: rec_type,
                    priority,
                    confidence: r.confidence,
                    title: r.title,
                    description: r.description,
                    impact_estimate: r.impact_estimate,
                    cost_impact: r.cost_impact.map(|c| rotappo_domain::CostImpact {
                        daily_change: c.daily_change,
                        currency: c.currency,
                    }),
                    action: r.action.and_then(|a| a.action).map(|a| match a {
                        rotappo_adapter_analytics::grpc::analytics::recommendation_action::Action::ScaleDeployment(s) => {
                            rotappo_domain::RecommendationAction::ScaleDeployment {
                                name: s.name,
                                from: s.from,
                                to: s.to,
                            }
                        }
                        rotappo_adapter_analytics::grpc::analytics::recommendation_action::Action::UpdateLimits(u) => {
                             rotappo_domain::RecommendationAction::UpdateResourceLimits {
                                resource: u.resource,
                                limits: rotappo_domain::ResourceLimits {
                                    cpu: u.limits.as_ref().and_then(|l| l.cpu.clone()),
                                    memory: u.limits.as_ref().and_then(|l| l.memory.clone()),
                                },
                            }
                        }
                        rotappo_adapter_analytics::grpc::analytics::recommendation_action::Action::ReclaimStorage(rs) => {
                            rotappo_domain::RecommendationAction::ReclaimStorage {
                                volume: rs.volume,
                                size_gb: rs.size_gb,
                            }
                        }
                    }).unwrap_or(rotappo_domain::RecommendationAction::ScaleDeployment {
                         name: "unknown".into(),
                         from: 0,
                         to: 0,
                    }),
                    status: r.status.and_then(|s| s.status).map(|s| match s {
                        rotappo_adapter_analytics::grpc::analytics::recommendation_status::Status::Pending(_) => {
                            rotappo_domain::RecommendationStatus::Pending
                        }
                        rotappo_adapter_analytics::grpc::analytics::recommendation_status::Status::ScheduledAt(t) => {
                            rotappo_domain::RecommendationStatus::Scheduled { execute_at: t }
                        }
                        rotappo_adapter_analytics::grpc::analytics::recommendation_status::Status::AppliedAt(t) => {
                            rotappo_domain::RecommendationStatus::Applied { applied_at: t }
                        }
                        rotappo_adapter_analytics::grpc::analytics::recommendation_status::Status::DismissedReason(reason) => {
                            rotappo_domain::RecommendationStatus::Dismissed { reason }
                        }
                    }).unwrap_or(rotappo_domain::RecommendationStatus::Pending),
                }
            })
            .collect::<Vec<_>>();

        Ok(domain_recs)
    }
}
