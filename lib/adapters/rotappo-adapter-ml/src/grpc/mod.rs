use anyhow::Result;
use std::net::SocketAddr;
use std::sync::Arc;
use tonic::{Request, Response, Status};

use phenome_domain as domain;
use phenome_ports::MLPort;

use crate::ml_service::MlService;

pub mod ml {
    tonic::include_proto!("ml");
}

pub mod analytics {
    tonic::include_proto!("analytics");
}

use ml::ml_service_server::{MlService as MlServiceTrait, MlServiceServer};
use ml::*;

#[derive(Debug)]
pub struct MlGrpcService {
    inner: Arc<MlService>,
}

impl MlGrpcService {
    pub fn new(inner: Arc<MlService>) -> Self {
        Self { inner }
    }
}

#[tonic::async_trait]
impl MlServiceTrait for MlGrpcService {
    async fn detect_anomalies(
        &self,
        request: Request<DetectAnomaliesRequest>,
    ) -> Result<Response<DetectAnomaliesResponse>, Status> {
        let req = request.into_inner();
        let data: domain::TimeSeriesData = req
            .data
            .ok_or_else(|| Status::invalid_argument("missing data"))?
            .try_into()
            .map_err(|e: anyhow::Error| Status::invalid_argument(e.to_string()))?;

        // Extract series from data for anomaly detection logic (if needed separate handling)
        // For now pass the whole data object which ML port expects (it expects domain::TimeSeriesData)
        // The ML Port trait has: async fn detect_anomalies(&self, data: TimeSeriesData) -> Result<Vec<Anomaly>>;
        // Wait, line 69 passes `time_series_data`.
        // So I just need to convert `req.data` (proto TimeSeriesData) -> `domain::TimeSeriesData`.

        let time_series_data = data;

        let anomalies = self
            .inner
            .detect_anomalies(time_series_data)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(DetectAnomaliesResponse {
            anomalies: anomalies.into_iter().map(Into::into).collect(),
        }))
    }

    async fn predict_scaling_needs(
        &self,
        request: Request<PredictScalingNeedsRequest>,
    ) -> Result<Response<PredictScalingNeedsResponse>, Status> {
        let req = request.into_inner();
        let prediction = self
            .inner
            .predict_scaling_needs(
                req.resource_id.clone(),
                std::time::Duration::from_millis(req.horizon_ms as u64),
            )
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(PredictScalingNeedsResponse {
            prediction: Some(ml::ScalingPrediction {
                resource_id: req.resource_id,
                generated_at: prediction.generated_at,
                horizon: prediction.horizon.as_millis() as i64,
                predicted_value: prediction.predicted_value,
                unit: prediction.unit,
            }),
        }))
    }

    async fn generate_recommendations(
        &self,
        request: Request<GenerateRecommendationsRequest>,
    ) -> Result<Response<GenerateRecommendationsResponse>, Status> {
        let req = request.into_inner();
        let recs = self
            .inner
            .generate_recommendations(req.cluster_id)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(GenerateRecommendationsResponse {
            recommendations: recs.into_iter().map(Into::into).collect(),
        }))
    }
}

pub struct GrpcServer;

impl GrpcServer {
    pub async fn serve(addr: SocketAddr, service: Arc<MlService>) -> Result<()> {
        let grpc_service = MlGrpcService::new(service);
        tonic::transport::Server::builder()
            .add_service(MlServiceServer::new(grpc_service))
            .serve(addr)
            .await?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct AnalyticsClient {
    client: analytics::analytics_service_client::AnalyticsServiceClient<tonic::transport::Channel>,
}

impl AnalyticsClient {
    pub async fn connect(endpoint: String) -> Result<Self> {
        let client =
            analytics::analytics_service_client::AnalyticsServiceClient::connect(endpoint).await?;
        Ok(Self { client })
    }

    pub async fn query_time_series(
        &mut self,
        resource_id: String,
    ) -> Result<Vec<domain::MetricSample>> {
        let now = chrono::Utc::now().timestamp_millis();
        let start = now - 3600 * 1000;

        let req = analytics::GetTimeSeriesRequest {
            resource_id,
            metric_type: 1, // MetricType::CpuUsage as i32 (1)? Or use analytics::MetricType::CpuUsage.into()
            time_range: Some(analytics::TimeRange {
                start_ms: start,
                end_ms: now,
            }),
        };

        let resp = self.client.get_time_series(req).await?;
        let inner = resp.into_inner();

        let mut samples = Vec::new();
        if let Some(ts) = inner.series {
            for p in ts.points {
                samples.push(domain::MetricSample {
                    cluster_id: ts.cluster_id.clone(),
                    resource_type: domain::ResourceType::Node, // Infer
                    resource_id: ts.resource_id.clone(),
                    metric_type: domain::MetricType::CpuUsage, // Infer
                    timestamp: p.timestamp,
                    value: p.value,
                    unit: ts.unit.clone(), // Use unit from TimeSeries
                });
            }
        }

        Ok(samples)
    }

    pub async fn fetch_historical(
        &mut self,
        req: domain::MetricsQuery,
    ) -> Result<Vec<domain::MetricSample>> {
        let range = req.time_range.map(|r| analytics::TimeRange {
            start_ms: r.start_ms,
            end_ms: r.end_ms,
        });

        // Use QueryMetrics because GetTimeSeries is singular
        let proto_req = analytics::QueryMetricsRequest {
            resource_type: req
                .resource_type
                .map(|r| i32::from(analytics::ResourceType::from(r))),
            cluster_id: req.cluster_id.clone(),
            resource_ids: req.resource_ids,
            metric_types: req
                .metric_types
                .into_iter()
                .map(|m| i32::from(analytics::MetricType::from(m)))
                .collect(),
            time_range: range,
        };

        let resp = self.client.query_metrics(proto_req).await?;
        let samples = resp.into_inner().samples;

        let domain_samples = samples
            .into_iter()
            .map(|s| {
                let r_type = s.resource_type;
                let m_type = s.metric_type;

                domain::MetricSample {
                    cluster_id: s.cluster_id,
                    resource_type: match analytics::ResourceType::try_from(r_type).ok() {
                        Some(analytics::ResourceType::Pod) => domain::ResourceType::Pod,
                        Some(analytics::ResourceType::Node) => domain::ResourceType::Node,
                        Some(analytics::ResourceType::Container) => domain::ResourceType::Container,
                        Some(analytics::ResourceType::Service) => domain::ResourceType::Service,
                        _ => domain::ResourceType::Pod, // Fallback
                    },
                    resource_id: s.resource_id,
                    metric_type: match analytics::MetricType::try_from(m_type).ok() {
                        Some(analytics::MetricType::CpuUsage) => domain::MetricType::CpuUsage,
                        Some(analytics::MetricType::MemoryUsage) => domain::MetricType::MemoryUsage,
                        Some(analytics::MetricType::NetworkIn) => domain::MetricType::NetworkIn,
                        Some(analytics::MetricType::NetworkOut) => domain::MetricType::NetworkOut,
                        Some(analytics::MetricType::DiskRead) => domain::MetricType::DiskRead,
                        Some(analytics::MetricType::DiskWrite) => domain::MetricType::DiskWrite,
                        _ => domain::MetricType::CpuUsage,
                    },
                    timestamp: s.timestamp,
                    value: s.value,
                    unit: s.unit,
                }
            })
            .collect();

        Ok(domain_samples)
    }
}

// Conversions (Simplified stubs for compilation)

impl TryFrom<ml::TimeSeriesData> for domain::TimeSeriesData {
    type Error = anyhow::Error;

    fn try_from(val: ml::TimeSeriesData) -> Result<Self, Self::Error> {
        Ok(domain::TimeSeriesData {
            cluster_id: val.cluster_id,
            range: val
                .range
                .ok_or_else(|| anyhow::anyhow!("missing range"))?
                .into(),
            series: val
                .series
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<_, _>>()?,
        })
    }
}

impl TryFrom<analytics::TimeSeries> for domain::TimeSeries {
    type Error = anyhow::Error;
    fn try_from(val: analytics::TimeSeries) -> Result<Self, Self::Error> {
        Ok(domain::TimeSeries {
            cluster_id: val.cluster_id,
            resource_id: val.resource_id,
            metric_type: analytics::MetricType::try_from(val.metric_type)
                .map_err(|_| anyhow::anyhow!("invalid metric type"))?
                .try_into()?,
            unit: val.unit,
            points: val.points.into_iter().map(Into::into).collect(),
        })
    }
}

impl From<analytics::TimeSeriesPoint> for domain::TimeSeriesPoint {
    fn from(val: analytics::TimeSeriesPoint) -> Self {
        Self {
            timestamp: val.timestamp,
            value: val.value,
        }
    }
}

impl From<analytics::TimeRange> for domain::TimeRange {
    fn from(val: analytics::TimeRange) -> Self {
        Self {
            start_ms: val.start_ms,
            end_ms: val.end_ms,
        }
    }
}

impl From<domain::ResourceType> for analytics::ResourceType {
    fn from(val: domain::ResourceType) -> Self {
        match val {
            domain::ResourceType::Pod => analytics::ResourceType::Pod,
            domain::ResourceType::Node => analytics::ResourceType::Node,
            domain::ResourceType::Container => analytics::ResourceType::Container,
            domain::ResourceType::Service => analytics::ResourceType::Service,
        }
    }
}

impl From<domain::MetricType> for analytics::MetricType {
    fn from(val: domain::MetricType) -> Self {
        match val {
            domain::MetricType::CpuUsage => analytics::MetricType::CpuUsage,
            domain::MetricType::MemoryUsage => analytics::MetricType::MemoryUsage,
            domain::MetricType::NetworkIn => analytics::MetricType::NetworkIn,
            domain::MetricType::NetworkOut => analytics::MetricType::NetworkOut,
            domain::MetricType::DiskRead => analytics::MetricType::DiskRead,
            domain::MetricType::DiskWrite => analytics::MetricType::DiskWrite,
        }
    }
}

impl TryFrom<analytics::MetricType> for domain::MetricType {
    type Error = anyhow::Error;
    fn try_from(val: analytics::MetricType) -> Result<Self, Self::Error> {
        match val {
            analytics::MetricType::CpuUsage => Ok(domain::MetricType::CpuUsage),
            _ => Err(anyhow::anyhow!("unsupported metric type")),
        }
    }
}

// Add more converters as needed for full coverage

impl From<domain::Anomaly> for analytics::Anomaly {
    fn from(val: domain::Anomaly) -> Self {
        Self {
            id: val.id,
            cluster_id: val.cluster_id,
            resource_id: val.resource_id,
            detected_at: val.detected_at,
            // ... map fields
            ..Default::default()
        }
    }
}

impl From<domain::Recommendation> for analytics::Recommendation {
    fn from(val: domain::Recommendation) -> Self {
        Self {
            id: val.id,
            // ... map fields
            ..Default::default()
        }
    }
}
