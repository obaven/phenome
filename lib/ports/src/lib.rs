use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use phenome_domain::{Assembly, Event, HealthSnapshot};

use async_trait::async_trait;

mod analytics;
mod notifications;
mod runtime;

pub use analytics::analytics::AnalyticsPort;
pub use analytics::metrics::MetricsPort;
pub use analytics::ml::MLPort;
pub use notifications::notification::NotificationPort;
pub use runtime::bootstrap::{
    AccessStatus, AccessUrlInfo, BootstrapPort, BootstrapStatus, ComponentState, ComponentStatus,
    ComponentTiming, InteractiveCommand,
};
pub use runtime::scheduler::SchedulerPort;

pub trait AssemblyPort: Send + Sync {
    fn assembly(&self) -> Option<Assembly>;
    fn assembly_error(&self) -> Option<String>;
    fn step_readiness(&self) -> std::collections::HashMap<String, bool>;
}

pub trait HealthPort: Send + Sync {
    fn snapshot(&self) -> HealthSnapshot;
}

pub trait LogPort: Send + Sync {
    fn drain_events(&self) -> Vec<Event>;
}

#[derive(Clone)]
pub struct PortSet {
    pub assembly: Arc<dyn AssemblyPort>,
    pub health: Arc<dyn HealthPort>,
    pub logs: Arc<dyn LogPort>,
    pub bootstrap: Arc<dyn BootstrapPort>,
    pub metrics: Arc<dyn MetricsPort>,
    pub analytics: Arc<dyn AnalyticsPort>,
    pub ml: Arc<dyn MLPort>,
    pub notifications: Arc<dyn NotificationPort>,
    pub scheduler: Arc<dyn SchedulerPort>,
}

impl PortSet {
    pub fn empty() -> Self {
        Self {
            assembly: Arc::new(NullAssemblyPort),
            health: Arc::new(NullHealthPort),
            logs: Arc::new(NullLogPort),
            bootstrap: Arc::new(NullBootstrapPort),
            metrics: Arc::new(NullMetricsPort),
            analytics: Arc::new(NullAnalyticsPort),
            ml: Arc::new(NullMLPort),
            notifications: Arc::new(NullNotificationPort),
            scheduler: Arc::new(NullSchedulerPort),
        }
    }
}

#[derive(Clone, Default)]
struct NullAssemblyPort;

impl AssemblyPort for NullAssemblyPort {
    fn assembly(&self) -> Option<Assembly> {
        None
    }

    fn assembly_error(&self) -> Option<String> {
        None
    }

    fn step_readiness(&self) -> std::collections::HashMap<String, bool> {
        std::collections::HashMap::new()
    }
}

#[derive(Clone, Default)]
struct NullHealthPort;

impl HealthPort for NullHealthPort {
    fn snapshot(&self) -> HealthSnapshot {
        HealthSnapshot::default()
    }
}

#[derive(Clone, Default)]
pub struct InMemoryLogPort {
    events: Arc<Mutex<VecDeque<Event>>>,
}

impl InMemoryLogPort {
    pub fn push(&self, event: Event) {
        if let Ok(mut guard) = self.events.lock() {
            guard.push_back(event);
        }
    }
}

impl LogPort for InMemoryLogPort {
    fn drain_events(&self) -> Vec<Event> {
        if let Ok(mut guard) = self.events.lock() {
            guard.drain(..).collect()
        } else {
            Vec::new()
        }
    }
}

#[derive(Clone, Default)]
struct NullLogPort;

impl LogPort for NullLogPort {
    fn drain_events(&self) -> Vec<Event> {
        Vec::new()
    }
}

#[derive(Clone, Default)]
struct NullBootstrapPort;

impl BootstrapPort for NullBootstrapPort {
    fn component_states(&self) -> std::collections::HashMap<String, ComponentState> {
        std::collections::HashMap::new()
    }

    fn dependency_graph(&self) -> &primer::domain::models::assembly::Assembly {
        static EMPTY: std::sync::OnceLock<primer::domain::models::assembly::Assembly> =
            std::sync::OnceLock::new();
        EMPTY.get_or_init(primer::domain::models::assembly::Assembly::default)
    }

    fn timing_history(&self) -> Option<primer::application::timing::TimingHistory> {
        None
    }

    fn bootstrap_status(&self) -> BootstrapStatus {
        BootstrapStatus::default()
    }

    fn access_urls(&self) -> Vec<AccessUrlInfo> {
        Vec::new()
    }

    fn send_command(
        &self,
        _cmd: primer::application::events::InteractiveCommand,
    ) -> anyhow::Result<()> {
        Ok(())
    }

    fn get_detailed_status(
        &self,
        _component_id: &str,
    ) -> anyhow::Result<primer::application::readiness::DetailedStatus> {
        Ok(primer::application::readiness::DetailedStatus::empty())
    }

    fn registry_specs(
        &self,
    ) -> std::collections::HashMap<String, primer::domain::models::module::spec::ModuleSpec>
    {
        std::collections::HashMap::new()
    }
}

#[derive(Clone, Default)]
struct NullMetricsPort;

#[async_trait]
impl MetricsPort for NullMetricsPort {
    async fn collect_metrics(
        &self,
        _cluster_id: phenome_domain::ClusterId,
    ) -> anyhow::Result<Vec<phenome_domain::MetricSample>> {
        Ok(Vec::new())
    }

    async fn query_metrics(
        &self,
        _query: phenome_domain::MetricsQuery,
    ) -> anyhow::Result<Vec<phenome_domain::MetricSample>> {
        Ok(Vec::new())
    }
}

#[derive(Clone, Default)]
struct NullAnalyticsPort;

#[async_trait]
impl AnalyticsPort for NullAnalyticsPort {
    async fn record_metrics(
        &self,
        _samples: Vec<phenome_domain::MetricSample>,
    ) -> anyhow::Result<()> {
        Ok(())
    }

    async fn query_aggregated(
        &self,
        _query: phenome_domain::AggregatedQuery,
    ) -> anyhow::Result<Vec<phenome_domain::AggregatedMetric>> {
        Ok(Vec::new())
    }

    async fn get_time_series(
        &self,
        resource_id: String,
        metric_type: phenome_domain::MetricType,
        _range: phenome_domain::TimeRange,
    ) -> anyhow::Result<phenome_domain::TimeSeries> {
        Ok(phenome_domain::TimeSeries {
            cluster_id: String::new(),
            resource_id,
            metric_type,
            unit: String::new(),
            points: Vec::new(),
        })
    }

    async fn get_anomalies(
        &self,
        _filter: phenome_domain::AnomalyFilter,
    ) -> anyhow::Result<Vec<phenome_domain::Anomaly>> {
        Ok(Vec::new())
    }

    async fn get_recommendations(
        &self,
        _filter: phenome_domain::RecommendationFilter,
    ) -> anyhow::Result<Vec<phenome_domain::Recommendation>> {
        Ok(Vec::new())
    }

    async fn query_metrics(
        &self,
        _query: phenome_domain::MetricsQuery,
    ) -> anyhow::Result<Vec<phenome_domain::MetricSample>> {
        Ok(Vec::new())
    }
}

#[derive(Clone, Default)]
struct NullMLPort;

#[async_trait]
impl MLPort for NullMLPort {
    async fn detect_anomalies(
        &self,
        _data: phenome_domain::TimeSeriesData,
    ) -> anyhow::Result<Vec<phenome_domain::Anomaly>> {
        Ok(Vec::new())
    }

    async fn predict_scaling_needs(
        &self,
        resource_id: String,
        horizon: std::time::Duration,
    ) -> anyhow::Result<phenome_domain::ScalingPrediction> {
        Ok(phenome_domain::ScalingPrediction {
            resource_id,
            generated_at: 0,
            horizon,
            predicted_value: 0.0,
            unit: String::new(),
        })
    }

    async fn generate_recommendations(
        &self,
        _cluster_id: phenome_domain::ClusterId,
    ) -> anyhow::Result<Vec<phenome_domain::Recommendation>> {
        Ok(Vec::new())
    }
}

#[derive(Clone, Default)]
struct NullNotificationPort;

#[async_trait]
impl NotificationPort for NullNotificationPort {
    async fn send_notification(
        &self,
        _notification: phenome_domain::Notification,
    ) -> anyhow::Result<()> {
        Ok(())
    }

    async fn configure_channel(
        &self,
        _channel: phenome_domain::NotificationChannel,
    ) -> anyhow::Result<()> {
        Ok(())
    }
}

#[derive(Clone, Default)]
struct NullSchedulerPort;

#[async_trait]
impl SchedulerPort for NullSchedulerPort {
    async fn schedule_action(
        &self,
        _action: phenome_domain::ScheduledAction,
    ) -> anyhow::Result<phenome_domain::ScheduleId> {
        Ok(String::new())
    }

    async fn cancel_schedule(&self, _id: phenome_domain::ScheduleId) -> anyhow::Result<()> {
        Ok(())
    }

    async fn list_scheduled(&self) -> anyhow::Result<Vec<phenome_domain::ScheduledAction>> {
        Ok(Vec::new())
    }
}
