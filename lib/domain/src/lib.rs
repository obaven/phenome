//! Domain models and invariants.

mod analytics;
mod infra;
mod ops;

pub use analytics::{anomaly, metrics, notification, recommendation};
pub use infra::{cluster, config, health};
pub use ops::{actions, assembly, events, snapshot};

pub use actions::{ActionDefinition, ActionId, ActionRegistry, ActionSafety};
pub use analytics::analytics::{
    AggregatedMetric, AggregatedQuery, MetricsQuery, ScalingPrediction, TimeRange, TimeSeries,
    TimeSeriesData, TimeSeriesPoint,
};
pub use analytics::anomaly::{Anomaly, AnomalyFilter, RootCauseAnalysis, Severity};
pub use assembly::{Assembly, AssemblyStepDef};
pub use cluster::{ClusterHealth, ClusterId, ClusterMetadata};
pub use config::{
    AnalyticsConfig, ClusterConfig, CollectionConfig, DeploymentConfig, MlConfig, MlModelsConfig,
    MlThresholdsConfig, NotificationChannelConfig, NotificationsConfig, RetentionConfig,
    RotappoConfig, ServicesConfig,
};
pub use events::{Event, EventBus, EventLevel};
pub use health::{ComponentHealthStatus, HealthSnapshot};
pub use metrics::{MetricSample, MetricType, ResourceType};
pub use notification::{Notification, NotificationChannel};
pub use recommendation::{
    CostImpact, Priority, Recommendation, RecommendationAction, RecommendationFilter,
    RecommendationStatus, RecommendationStatusKind, RecommendationType, ResourceLimits, ScheduleId,
    ScheduleStatus, ScheduledAction,
};
pub use snapshot::{
    ActionStatus, AssemblyStep, AssemblyStepStatus, AssemblySummary, Capability, CapabilityStatus,
    HealthStatus, Snapshot, now_millis,
};
