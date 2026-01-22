use std::collections::HashMap;
use std::time::{Duration, Instant};

use anyhow::Result;

pub use primer::application::events::InteractiveCommand;
use primer::application::readiness::{DetailedStatus, ReadinessStatus};
use primer::application::timing::TimingHistory;
use primer::domain::models::assembly::Assembly;
use primer::domain::models::module::spec::ModuleSpec;

#[derive(Debug, Clone)]
pub struct ComponentState {
    pub id: String,
    pub status: ComponentStatus,
    pub readiness: Option<ReadinessStatus>,
    pub timing: ComponentTiming,
    pub retry_count: u32,
    pub deferred_reason: Option<String>,
}

impl ComponentState {
    pub fn new(id: String) -> Self {
        Self {
            id,
            status: ComponentStatus::Pending,
            readiness: None,
            timing: ComponentTiming::default(),
            retry_count: 0,
            deferred_reason: None,
        }
    }

    pub fn mark_running(&mut self, started_at: Instant) {
        self.status = ComponentStatus::Running;
        self.timing.started_at = Some(started_at);
    }

    pub fn mark_completed(&mut self, duration: Duration) {
        self.status = ComponentStatus::Complete;
        self.timing.total_duration = Some(duration);
        self.timing.completed_at = Some(Instant::now());
    }

    pub fn mark_failed(&mut self, duration: Duration) {
        self.status = ComponentStatus::Failed;
        self.timing.total_duration = Some(duration);
        self.timing.completed_at = Some(Instant::now());
    }

    pub fn mark_deferred(&mut self, reason: String) {
        self.status = ComponentStatus::Deferred;
        self.deferred_reason = Some(reason);
        self.timing.completed_at = Some(Instant::now());
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComponentStatus {
    Pending,
    Running,
    Complete,
    Failed,
    Deferred,
}

#[derive(Debug, Clone, Default)]
pub struct ComponentTiming {
    pub started_at: Option<Instant>,
    pub completed_at: Option<Instant>,
    pub total_duration: Option<Duration>,
    pub render_duration: Option<Duration>,
    pub apply_duration: Option<Duration>,
    pub wait_duration: Option<Duration>,
    pub last_elapsed: Option<Duration>,
}

impl ComponentTiming {
    pub fn update_elapsed(&mut self, elapsed: Duration) {
        self.last_elapsed = Some(elapsed);
    }

    pub fn current_elapsed(&self) -> Option<Duration> {
        self.last_elapsed
            .or_else(|| self.started_at.map(|started| started.elapsed()))
    }
}

#[derive(Debug, Clone, Default)]
pub struct BootstrapStatus {
    pub started_at: Option<Instant>,
    pub total_duration: Option<Duration>,
    pub total_components: Option<usize>,
    pub successful: usize,
    pub failed: usize,
    pub deferred: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccessStatus {
    Pending,
    Ready,
    Unknown,
}

impl AccessStatus {
    pub fn label(self) -> &'static str {
        match self {
            AccessStatus::Pending => "Pending",
            AccessStatus::Ready => "Ready",
            AccessStatus::Unknown => "Unknown",
        }
    }
}

#[derive(Debug, Clone)]
pub struct AccessUrlInfo {
    pub service: String,
    pub url: String,
    pub status: AccessStatus,
}

pub trait BootstrapPort: Send + Sync {
    fn component_states(&self) -> HashMap<String, ComponentState>;
    fn dependency_graph(&self) -> &Assembly;
    fn timing_history(&self) -> Option<TimingHistory>;
    fn bootstrap_status(&self) -> BootstrapStatus;
    fn access_urls(&self) -> Vec<AccessUrlInfo>;
    fn send_command(&self, cmd: InteractiveCommand) -> Result<()>;
    fn get_detailed_status(&self, component_id: &str) -> Result<DetailedStatus>;
    fn registry_specs(&self) -> HashMap<String, ModuleSpec>;
}
