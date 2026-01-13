use anyhow::Result;
use std::net::SocketAddr;
use std::sync::Arc;
use tonic::{Request, Response, Status};

use rotappo_domain as domain;
use rotappo_ports::AnalyticsPort;

use crate::AnalyticsService;

pub mod analytics {
    tonic::include_proto!("analytics");
}

use analytics::analytics_service_server::{
    AnalyticsService as AnalyticsServiceTrait, AnalyticsServiceServer,
};
use analytics::*;

#[derive(Debug)]
pub struct GrpcAnalyticsService {
    inner: Arc<AnalyticsService>,
}

impl GrpcAnalyticsService {
    pub fn new(inner: Arc<AnalyticsService>) -> Self {
        Self { inner }
    }
}

#[tonic::async_trait]
impl AnalyticsServiceTrait for GrpcAnalyticsService {
    async fn record_metrics(
        &self,
        request: Request<RecordMetricsRequest>,
    ) -> Result<Response<RecordMetricsResponse>, Status> {
        let req = request.into_inner();
        let samples: Vec<domain::MetricSample> = req
            .samples
            .into_iter()
            .map(TryInto::try_into)
            .collect::<Result<_, _>>()
            .map_err(|e: anyhow::Error| Status::invalid_argument(e.to_string()))?;

        self.inner
            .record_metrics(samples)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(RecordMetricsResponse {}))
    }

    async fn query_aggregated(
        &self,
        request: Request<QueryAggregatedRequest>,
    ) -> Result<Response<QueryAggregatedResponse>, Status> {
        let req = request.into_inner();
        let query: domain::AggregatedQuery = req
            .try_into()
            .map_err(|e: anyhow::Error| Status::invalid_argument(e.to_string()))?;

        let metrics = self
            .inner
            .query_aggregated(query)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(QueryAggregatedResponse {
            metrics: metrics.into_iter().map(Into::into).collect(),
        }))
    }

    async fn get_time_series(
        &self,
        request: Request<GetTimeSeriesRequest>,
    ) -> Result<Response<GetTimeSeriesResponse>, Status> {
        let req = request.into_inner();
        let metric_type = req
            .metric_type
            .try_into() // i32
            .map_err(|_| Status::invalid_argument("Invalid metric type"))
            .and_then(|t| {
                MetricType::from_i32(t)
                    .ok_or_else(|| Status::invalid_argument("Invalid metric type"))
            })?
            .try_into()
            .map_err(|e: anyhow::Error| Status::invalid_argument(e.to_string()))?;
        let range = req
            .time_range
            .ok_or_else(|| Status::invalid_argument("missing time range"))?
            .into();

        let series = self
            .inner
            .get_time_series(req.resource_id, metric_type, range)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(GetTimeSeriesResponse {
            series: Some(series.into()),
        }))
    }

    async fn get_anomalies(
        &self,
        request: Request<GetAnomaliesRequest>,
    ) -> Result<Response<GetAnomaliesResponse>, Status> {
        let req = request.into_inner();
        let filter: domain::AnomalyFilter = req.into();

        let anomalies = self
            .inner
            .get_anomalies(filter)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(GetAnomaliesResponse {
            anomalies: anomalies.into_iter().map(Into::into).collect(),
        }))
    }

    async fn get_recommendations(
        &self,
        request: Request<GetRecommendationsRequest>,
    ) -> Result<Response<GetRecommendationsResponse>, Status> {
        let req = request.into_inner();
        let filter: domain::RecommendationFilter = req.into();

        let recommendations = self
            .inner
            .get_recommendations(filter)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(GetRecommendationsResponse {
            recommendations: recommendations.into_iter().map(Into::into).collect(),
        }))
    }

    async fn query_metrics(
        &self,
        request: Request<QueryMetricsRequest>,
    ) -> Result<Response<QueryMetricsResponse>, Status> {
        let req = request.into_inner();
        let query: domain::MetricsQuery = req.into();

        let samples = self
            .inner
            .query_metrics(query)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(QueryMetricsResponse {
            samples: samples.into_iter().map(Into::into).collect(),
        }))
    }
}

pub struct GrpcServer;

impl GrpcServer {
    pub async fn serve(addr: SocketAddr, service: Arc<AnalyticsService>) -> Result<()> {
        let grpc_service = GrpcAnalyticsService::new(service);
        tonic::transport::Server::builder()
            .add_service(AnalyticsServiceServer::new(grpc_service))
            .serve(addr)
            .await?;
        Ok(())
    }
}

pub mod ml {
    tonic::include_proto!("ml");
}

#[derive(Debug, Clone)]
pub struct MlClient {
    // In a real app, this should be a pool or a robust client wrapper
    // For now, storing the endpoint to connect on demand or a channel if established
    endpoint: String,
}

impl MlClient {
    pub async fn connect(endpoint: &str) -> Result<Self> {
        // Just store endpoint, connection happens on request or we could establish channel here
        // verifying connectivity
        Ok(Self {
            endpoint: endpoint.to_string(),
        })
    }

    pub async fn detect_anomalies(
        &self,
        series: &domain::TimeSeries,
    ) -> Result<Vec<domain::Anomaly>> {
        let mut client =
            ml::ml_service_client::MlServiceClient::connect(self.endpoint.clone()).await?;

        // Convert domain TimeSeries to proto TimeSeries
        // We need a helper or From/TryFrom implementation
        // For simplicity, constructing request manually or using a stub conversion if complex

        let proto_series = analytics::TimeSeries {
            // Simplified stub mapping
            cluster_id: series.cluster_id.clone(),
            resource_id: series.resource_id.clone(),
            metric_type: i32::from(analytics::MetricType::from(series.metric_type)),
            unit: series.unit.clone(),
            points: series
                .points
                .iter()
                .map(|p| analytics::TimeSeriesPoint {
                    timestamp: p.timestamp,
                    value: p.value,
                })
                .collect(),
        };

        let range = if let (Some(first), Some(last)) = (series.points.first(), series.points.last())
        {
            analytics::TimeRange {
                start_ms: first.timestamp,
                end_ms: last.timestamp,
            }
        } else {
            analytics::TimeRange {
                start_ms: 0,
                end_ms: 0,
            }
        };

        let time_series_data = ml::TimeSeriesData {
            cluster_id: series.cluster_id.clone(),
            range: Some(range),
            series: vec![proto_series],
        };

        let request = tonic::Request::new(ml::DetectAnomaliesRequest {
            data: Some(time_series_data),
        });

        let response = client.detect_anomalies(request).await?;
        let inner = response.into_inner();

        // Convert back
        // Stub conversion
        Ok(inner
            .anomalies
            .into_iter()
            .map(|a| domain::Anomaly {
                id: a.id,
                cluster_id: a.cluster_id,
                resource_id: a.resource_id,
                detected_at: a.detected_at,
                metric_type: match analytics::MetricType::from_i32(a.metric_type) {
                    Some(analytics::MetricType::CpuUsage) => domain::MetricType::CpuUsage,
                    Some(analytics::MetricType::MemoryUsage) => domain::MetricType::MemoryUsage,
                    Some(analytics::MetricType::NetworkIn) => domain::MetricType::NetworkIn,
                    Some(analytics::MetricType::NetworkOut) => domain::MetricType::NetworkOut,
                    Some(analytics::MetricType::DiskRead) => domain::MetricType::DiskRead,
                    Some(analytics::MetricType::DiskWrite) => domain::MetricType::DiskWrite,
                    _ => domain::MetricType::CpuUsage, // Fallback
                },
                severity: match analytics::Severity::try_from(a.severity) {
                    Ok(analytics::Severity::Critical) => domain::Severity::Critical,
                    Ok(analytics::Severity::Warning) => domain::Severity::Warning,
                    Ok(analytics::Severity::Info) => domain::Severity::Info,
                    _ => domain::Severity::Info,
                },
                confidence: a.confidence,
                description: a.description,
                baseline_value: a.baseline_value,
                observed_value: a.observed_value,
                deviation_sigma: a.deviation_sigma,
                related_metrics: a.related_metrics,
                root_cause: a.root_cause.filter(|s| !s.is_empty()),
            })
            .collect())
    }
}

// Conversions

impl TryFrom<MetricSample> for domain::MetricSample {
    type Error = anyhow::Error;

    fn try_from(val: MetricSample) -> Result<Self, Self::Error> {
        let resource_type = match ResourceType::try_from(val.resource_type)? {
            analytics::ResourceType::Pod => domain::ResourceType::Pod,
            analytics::ResourceType::Node => domain::ResourceType::Node,
            analytics::ResourceType::Container => domain::ResourceType::Container,
            analytics::ResourceType::Service => domain::ResourceType::Service,
            _ => domain::ResourceType::Pod,
        };

        let metric_type = match MetricType::try_from(val.metric_type)? {
            analytics::MetricType::CpuUsage => domain::MetricType::CpuUsage,
            analytics::MetricType::MemoryUsage => domain::MetricType::MemoryUsage,
            analytics::MetricType::NetworkIn => domain::MetricType::NetworkIn,
            analytics::MetricType::NetworkOut => domain::MetricType::NetworkOut,
            analytics::MetricType::DiskRead => domain::MetricType::DiskRead,
            analytics::MetricType::DiskWrite => domain::MetricType::DiskWrite,
            _ => domain::MetricType::CpuUsage,
        };

        Ok(domain::MetricSample {
            cluster_id: val.cluster_id,
            resource_type,
            resource_id: val.resource_id,
            metric_type,
            timestamp: val.timestamp,
            value: val.value,
            unit: val.unit,
        })
    }
}

impl From<domain::MetricSample> for MetricSample {
    fn from(val: domain::MetricSample) -> Self {
        Self {
            cluster_id: val.cluster_id,
            resource_type: ResourceType::from(val.resource_type).into(),
            resource_id: val.resource_id,
            metric_type: MetricType::from(val.metric_type).into(),
            timestamp: val.timestamp,
            value: val.value,
            unit: val.unit,
        }
    }
}

impl TryFrom<ResourceType> for domain::ResourceType {
    type Error = anyhow::Error;

    fn try_from(val: ResourceType) -> Result<Self, Self::Error> {
        match val {
            ResourceType::Pod => Ok(domain::ResourceType::Pod),
            ResourceType::Node => Ok(domain::ResourceType::Node),
            ResourceType::Container => Ok(domain::ResourceType::Container),
            ResourceType::Service => Ok(domain::ResourceType::Service),
            ResourceType::Unspecified => anyhow::bail!("unspecified resource type"),
        }
    }
}

impl From<domain::ResourceType> for ResourceType {
    fn from(val: domain::ResourceType) -> Self {
        match val {
            domain::ResourceType::Pod => ResourceType::Pod,
            domain::ResourceType::Node => ResourceType::Node,
            domain::ResourceType::Container => ResourceType::Container,
            domain::ResourceType::Service => ResourceType::Service,
        }
    }
}

impl TryFrom<MetricType> for domain::MetricType {
    type Error = anyhow::Error;

    fn try_from(val: MetricType) -> Result<Self, Self::Error> {
        match val {
            MetricType::CpuUsage => Ok(domain::MetricType::CpuUsage),
            MetricType::MemoryUsage => Ok(domain::MetricType::MemoryUsage),
            MetricType::NetworkIn => Ok(domain::MetricType::NetworkIn),
            MetricType::NetworkOut => Ok(domain::MetricType::NetworkOut),
            MetricType::DiskRead => Ok(domain::MetricType::DiskRead),
            MetricType::DiskWrite => Ok(domain::MetricType::DiskWrite),
            MetricType::Unspecified => anyhow::bail!("unspecified metric type"),
        }
    }
}

impl From<domain::MetricType> for MetricType {
    fn from(val: domain::MetricType) -> Self {
        match val {
            domain::MetricType::CpuUsage => MetricType::CpuUsage,
            domain::MetricType::MemoryUsage => MetricType::MemoryUsage,
            domain::MetricType::NetworkIn => MetricType::NetworkIn,
            domain::MetricType::NetworkOut => MetricType::NetworkOut,
            domain::MetricType::DiskRead => MetricType::DiskRead,
            domain::MetricType::DiskWrite => MetricType::DiskWrite,
        }
    }
}

impl TryFrom<QueryAggregatedRequest> for domain::AggregatedQuery {
    type Error = anyhow::Error;

    fn try_from(val: QueryAggregatedRequest) -> Result<Self, Self::Error> {
        Ok(domain::AggregatedQuery {
            cluster_id: val.cluster_id,
            resource_type: val
                .resource_type
                .map(|t| {
                    ResourceType::from_i32(t)
                        .ok_or_else(|| anyhow::anyhow!("Invalid resource type"))
                })
                .transpose()?
                .map(|t| t.try_into())
                .transpose()?,
            metric_types: val
                .metric_types
                .into_iter()
                .map(|t| {
                    let mt = MetricType::from_i32(t)
                        .ok_or_else(|| anyhow::anyhow!("Invalid metric type"))?;
                    mt.try_into()
                })
                .collect::<Result<_, _>>()?,
            window_duration: std::time::Duration::from_millis(val.window_duration_ms as u64),
            time_range: val.time_range.map(Into::into),
        })
    }
}

impl From<domain::AggregatedMetric> for AggregatedMetric {
    fn from(val: domain::AggregatedMetric) -> Self {
        Self {
            cluster_id: val.cluster_id,
            resource_type: ResourceType::from(val.resource_type).into(),
            metric_type: MetricType::from(val.metric_type).into(),
            window_start: val.window_start,
            window_duration_ms: val.window_duration.as_millis() as i64,
            count: val.count,
            sum: val.sum,
            min: val.min,
            max: val.max,
            avg: val.avg,
            p50: val.p50,
            p95: val.p95,
            p99: val.p99,
        }
    }
}

impl From<TimeRange> for domain::TimeRange {
    fn from(val: TimeRange) -> Self {
        domain::TimeRange {
            start_ms: val.start_ms,
            end_ms: val.end_ms,
        }
    }
}

impl From<domain::TimeRange> for TimeRange {
    fn from(val: domain::TimeRange) -> Self {
        TimeRange {
            start_ms: val.start_ms,
            end_ms: val.end_ms,
        }
    }
}

impl From<domain::TimeSeries> for TimeSeries {
    fn from(val: domain::TimeSeries) -> Self {
        Self {
            cluster_id: val.cluster_id,
            resource_id: val.resource_id,
            metric_type: MetricType::from(val.metric_type).into(),
            unit: val.unit,
            points: val.points.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<domain::TimeSeriesPoint> for TimeSeriesPoint {
    fn from(val: domain::TimeSeriesPoint) -> Self {
        Self {
            timestamp: val.timestamp,
            value: val.value,
        }
    }
}

impl From<GetAnomaliesRequest> for domain::AnomalyFilter {
    fn from(val: GetAnomaliesRequest) -> Self {
        domain::AnomalyFilter {
            cluster_id: val.cluster_id,
            resource_id: val.resource_id,
            metric_type: val
                .metric_type
                .and_then(|t| MetricType::from_i32(t).and_then(|t| t.try_into().ok())),
            severity: val
                .severity
                .and_then(|s| Severity::from_i32(s).and_then(|s| s.try_into().ok())),
            time_range: val.time_range.map(Into::into),
            limit: val.limit,
        }
    }
}

impl From<domain::Anomaly> for Anomaly {
    fn from(val: domain::Anomaly) -> Self {
        Self {
            id: val.id,
            cluster_id: val.cluster_id,
            resource_id: val.resource_id,
            detected_at: val.detected_at,
            metric_type: MetricType::from(val.metric_type).into(),
            severity: Severity::from(val.severity).into(),
            confidence: val.confidence,
            description: val.description,
            baseline_value: val.baseline_value,
            observed_value: val.observed_value,
            deviation_sigma: val.deviation_sigma,
            related_metrics: val.related_metrics,
            root_cause: val.root_cause,
        }
    }
}

impl TryFrom<Severity> for domain::Severity {
    type Error = anyhow::Error;

    fn try_from(val: Severity) -> Result<Self, Self::Error> {
        match val {
            Severity::Critical => Ok(domain::Severity::Critical),
            Severity::Warning => Ok(domain::Severity::Warning),
            Severity::Info => Ok(domain::Severity::Info),
            Severity::Unspecified => anyhow::bail!("unspecified severity"),
        }
    }
}

impl From<domain::Severity> for Severity {
    fn from(val: domain::Severity) -> Self {
        match val {
            domain::Severity::Critical => Severity::Critical,
            domain::Severity::Warning => Severity::Warning,
            domain::Severity::Info => Severity::Info,
        }
    }
}

impl From<GetRecommendationsRequest> for domain::RecommendationFilter {
    fn from(val: GetRecommendationsRequest) -> Self {
        domain::RecommendationFilter {
            cluster_id: val.cluster_id,
            priority: val
                .priority
                .and_then(|p| Priority::try_from(p).ok().and_then(|p| p.try_into().ok())),
            status: val.status.and_then(|s| {
                RecommendationStatusKind::try_from(s)
                    .ok()
                    .and_then(|s| s.try_into().ok())
            }),
            limit: val.limit,
        }
    }
}

impl From<domain::Recommendation> for Recommendation {
    fn from(val: domain::Recommendation) -> Self {
        Self {
            id: val.id,
            cluster_id: val.cluster_id,
            created_at: val.created_at,
            recommendation_type: RecommendationType::from(val.recommendation_type).into(),
            priority: Priority::from(val.priority).into(),
            confidence: val.confidence,
            title: val.title,
            description: val.description,
            impact_estimate: val.impact_estimate,
            cost_impact: val.cost_impact.map(|c| CostImpact {
                daily_change: c.daily_change,
                currency: c.currency,
            }),
            action: Some(match val.action {
                domain::RecommendationAction::ScaleDeployment { name, from, to } => {
                    RecommendationAction {
                        action: Some(recommendation_action::Action::ScaleDeployment(
                            ScaleDeploymentAction { name, from, to },
                        )),
                    }
                }
                domain::RecommendationAction::UpdateResourceLimits { resource, limits } => {
                    RecommendationAction {
                        action: Some(recommendation_action::Action::UpdateLimits(
                            UpdateResourceLimitsAction {
                                resource,
                                limits: Some(ResourceLimits {
                                    cpu: limits.cpu,
                                    memory: limits.memory,
                                }),
                            },
                        )),
                    }
                }
                domain::RecommendationAction::ReclaimStorage { volume, size_gb } => {
                    RecommendationAction {
                        action: Some(recommendation_action::Action::ReclaimStorage(
                            ReclaimStorageAction { volume, size_gb },
                        )),
                    }
                }
            }),
            status: Some(match val.status {
                domain::RecommendationStatus::Pending => RecommendationStatus {
                    status: Some(recommendation_status::Status::Pending(true)),
                },
                domain::RecommendationStatus::Scheduled { execute_at } => RecommendationStatus {
                    status: Some(recommendation_status::Status::ScheduledAt(execute_at)),
                },
                domain::RecommendationStatus::Applied { applied_at } => RecommendationStatus {
                    status: Some(recommendation_status::Status::AppliedAt(applied_at)),
                },
                domain::RecommendationStatus::Dismissed { reason } => RecommendationStatus {
                    status: Some(recommendation_status::Status::DismissedReason(reason)),
                },
            }),
        }
    }
}

impl TryFrom<Priority> for domain::Priority {
    type Error = anyhow::Error;

    fn try_from(val: Priority) -> Result<Self, Self::Error> {
        match val {
            Priority::High => Ok(domain::Priority::High),
            Priority::Medium => Ok(domain::Priority::Medium),
            Priority::Low => Ok(domain::Priority::Low),
            Priority::Unspecified => anyhow::bail!("unspecified priority"),
        }
    }
}

impl From<domain::Priority> for Priority {
    fn from(val: domain::Priority) -> Self {
        match val {
            domain::Priority::High => Priority::High,
            domain::Priority::Medium => Priority::Medium,
            domain::Priority::Low => Priority::Low,
        }
    }
}

impl TryFrom<RecommendationType> for domain::RecommendationType {
    type Error = anyhow::Error;

    fn try_from(val: RecommendationType) -> Result<Self, Self::Error> {
        match val {
            RecommendationType::ScaleUp => Ok(domain::RecommendationType::ScaleUp),
            RecommendationType::ScaleDown => Ok(domain::RecommendationType::ScaleDown),
            RecommendationType::OptimizeResources => {
                Ok(domain::RecommendationType::OptimizeResources)
            }
            RecommendationType::AdjustLimits => Ok(domain::RecommendationType::AdjustLimits),
            RecommendationType::StorageOptimizations => {
                Ok(domain::RecommendationType::StorageOptimization)
            }
            RecommendationType::Unspecified => anyhow::bail!("unspecified recommendation type"),
        }
    }
}

impl From<domain::RecommendationType> for RecommendationType {
    fn from(val: domain::RecommendationType) -> Self {
        match val {
            domain::RecommendationType::ScaleUp => RecommendationType::ScaleUp,
            domain::RecommendationType::ScaleDown => RecommendationType::ScaleDown,
            domain::RecommendationType::OptimizeResources => RecommendationType::OptimizeResources,
            domain::RecommendationType::AdjustLimits => RecommendationType::AdjustLimits,
            domain::RecommendationType::StorageOptimization => {
                RecommendationType::StorageOptimizations
            }
        }
    }
}

impl TryFrom<RecommendationStatusKind> for domain::RecommendationStatusKind {
    type Error = anyhow::Error;

    fn try_from(val: RecommendationStatusKind) -> Result<Self, Self::Error> {
        match val {
            RecommendationStatusKind::Pending => Ok(domain::RecommendationStatusKind::Pending),
            RecommendationStatusKind::Scheduled => Ok(domain::RecommendationStatusKind::Scheduled),
            RecommendationStatusKind::Applied => Ok(domain::RecommendationStatusKind::Applied),
            RecommendationStatusKind::Dismissed => Ok(domain::RecommendationStatusKind::Dismissed),
            RecommendationStatusKind::Unspecified => {
                anyhow::bail!("unspecified recommendation status kind")
            }
        }
    }
}

impl From<QueryMetricsRequest> for domain::MetricsQuery {
    fn from(val: QueryMetricsRequest) -> Self {
        domain::MetricsQuery {
            cluster_id: val.cluster_id,
            resource_type: val.resource_type.and_then(|t| {
                ResourceType::try_from(t)
                    .ok()
                    .and_then(|t| t.try_into().ok())
            }),
            resource_ids: val.resource_ids,
            metric_types: val
                .metric_types
                .into_iter()
                .filter_map(|t| MetricType::try_from(t).ok().and_then(|t| t.try_into().ok()))
                .collect(),
            time_range: val.time_range.map(Into::into),
        }
    }
}
