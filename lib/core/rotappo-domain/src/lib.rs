//! Domain models and invariants.

pub mod actions;
pub mod analytics;
pub mod anomaly;
pub mod assembly;
pub mod cluster;
pub mod config;
pub mod events;
pub mod health;
pub mod metrics;
pub mod notification;
pub mod recommendation;
pub mod snapshot;

pub use actions::{ActionDefinition, ActionId, ActionRegistry, ActionSafety};
pub use analytics::{
    AggregatedMetric, AggregatedQuery, MetricsQuery, ScalingPrediction, TimeRange, TimeSeries,
    TimeSeriesData, TimeSeriesPoint,
};
pub use anomaly::{Anomaly, AnomalyFilter, RootCauseAnalysis, Severity};
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
