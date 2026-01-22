//! Bootstrappo adapter: event stream -> UI state projection.
//!
//! ## Responsibility
//! - Subscribe to the Bootstrappo EventBus.
//! - Maintain component state and bootstrap summary for the TUI.
//! - Provide access to dependency graphs, timing history, and access URLs.
//!
//! ## Non-goals
//! - Execute bootstrap actions or cluster init logic.
//! - Persist timing history beyond the underlying storage adapters.
//!
//! ## Key invariants
//! - Event processing is best-effort and must never panic the UI.
//! - Component state transitions are monotonic (Pending -> Running -> Complete/Failed/Deferred).
//!
//! ## Failure modes
//! - Event stream lag or dropped events (lossy by design).
//! - K8s API failures during access URL discovery (ignored until next run).
//!
//! ## Performance notes
//! - State map updates are O(1) per event.
//! - Cache reads are O(1) with TTL bounds.
//!
//! ## Security / safety notes
//! - No secrets are stored in the adapter state.
//!
//! ## Extension points
//! - Add cluster init event projections for pre-reconcile visibility.
//! - Add richer access URL health checks.
use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, Instant};

use anyhow::{Context, Result, anyhow};
use k8s_openapi::api::networking::v1::Ingress;
use kube::api::ListParams;
use tokio::sync::mpsc;

use primer::adapters::infrastructure::kube::clients::k8s::K8sClient;
use primer::application::events::{
    BootstrapEvent, EventBus, EventPayload, InteractiveCommand,
};
use primer::application::readiness::{DetailedStatus, map_check_to_readiness};
use primer::application::timing::TimingHistory;
use primer::application::timing::storage::{
    cluster::ClusterTimingStorage, local::LocalTimingStorage,
};
use primer::domain::models::assembly::Assembly;
use primer::domain::models::module::spec::ModuleSpec;

use phenome_ports::{
    AccessStatus, AccessUrlInfo, BootstrapPort, BootstrapStatus, ComponentState, ComponentStatus,
};

const CACHE_TTL: Duration = Duration::from_secs(5);

/// Adapter that projects Bootstrappo events into TUI-ready state.
///
/// ## Why
/// Decouple UI rendering from the bootstrap engine by maintaining a local
/// snapshot of component state and summary metrics.
///
/// ## Inputs
/// - EventBus stream for bootstrap events.
/// - Assembly definition for dependency graphs.
/// - Command channel for interactive commands.
/// - K8s client for detailed status and access URL discovery.
///
/// ## Output
/// - Implements `BootstrapPort` for TUI consumption.
///
/// ## Invariants
/// - State map always contains at least the assembly-defined component IDs.
///
/// ## Complexity
/// - Time: O(1) per event update.
/// - Space: O(N) for component state.
pub struct BootstrapAdapter {
    state: Arc<RwLock<HashMap<String, ComponentState>>>,
    assembly: Arc<Assembly>,
    timing_history: Arc<RwLock<Option<TimingHistory>>>,
    command_tx: mpsc::Sender<InteractiveCommand>,
    detailed_cache: Arc<Mutex<DetailedStatusCache>>,
    status: Arc<RwLock<BootstrapStatus>>,
    access_urls: Arc<RwLock<Vec<AccessUrlInfo>>>,
    k8s: K8sClient,
}

impl BootstrapAdapter {
    /// Construct a new adapter and spawn the event listener.
    ///
    /// ## Why
    /// Centralize event processing and shared caches for the TUI.
    ///
    /// ## Inputs
    /// - `event_bus`: EventBus to subscribe to.
    /// - `assembly`: Assembly definition for dependency graph access.
    /// - `command_tx`: Command channel back to the reconciler.
    /// - `k8s`: K8s client used for detailed status.
    ///
    /// ## Output
    /// - Returns a ready-to-use adapter instance.
    ///
    /// ## Invariants
    /// - Spawns a background task to process events.
    ///
    /// ## Complexity
    /// - Time: O(N) to seed initial component state.
    pub fn new(
        event_bus: EventBus,
        assembly: Assembly,
        command_tx: mpsc::Sender<InteractiveCommand>,
        k8s: K8sClient,
    ) -> Self {
        let mut state_map = HashMap::new();
        for step in &assembly.steps {
            state_map.insert(step.id.clone(), ComponentState::new(step.id.clone()));
        }

        let state = Arc::new(RwLock::new(state_map));
        let status = Arc::new(RwLock::new(BootstrapStatus::default()));
        let timing_history = Arc::new(RwLock::new(None));
        let detailed_cache = Arc::new(Mutex::new(DetailedStatusCache::new(CACHE_TTL)));
        let assembly = Arc::new(assembly);
        let access_urls = Arc::new(RwLock::new(Vec::new()));

        let adapter = Self {
            state: Arc::clone(&state),
            assembly: Arc::clone(&assembly),
            timing_history: Arc::clone(&timing_history),
            command_tx,
            detailed_cache: Arc::clone(&detailed_cache),
            status: Arc::clone(&status),
            access_urls: Arc::clone(&access_urls),
            k8s,
        };

        adapter.spawn_event_listener(
            event_bus,
            state,
            status,
            timing_history,
            detailed_cache,
            access_urls,
        );

        adapter
    }

    fn spawn_event_listener(
        &self,
        event_bus: EventBus,
        state: Arc<RwLock<HashMap<String, ComponentState>>>,
        status: Arc<RwLock<BootstrapStatus>>,
        timing_history: Arc<RwLock<Option<TimingHistory>>>,
        detailed_cache: Arc<Mutex<DetailedStatusCache>>,
        access_urls: Arc<RwLock<Vec<AccessUrlInfo>>>,
    ) {
        let k8s = self.k8s.clone();
        tokio::spawn(async move {
            // Initial fetch of access URLs (in case bootstrap is already done or ongoing)
            if let Ok(urls) = Self::fetch_access_urls(&k8s).await {
                if let Ok(mut guard) = access_urls.write() {
                    *guard = urls;
                }
            }
            // Initial fetch of timing history
            if let Ok(history) = Self::load_timing_history().await {
                if let Ok(mut guard) = timing_history.write() {
                    *guard = Some(history);
                }
            }

            let mut rx = event_bus.subscribe();
            while let Ok(event) = rx.recv().await {
                Self::process_event(&event, &state, &status, &detailed_cache);

                if matches!(event.payload, EventPayload::Completed { .. }) {
                    if let Ok(urls) = Self::fetch_access_urls(&k8s).await {
                        if let Ok(mut guard) = access_urls.write() {
                            *guard = urls;
                        }
                    }
                    if let Ok(history) = Self::load_timing_history().await {
                        if let Ok(mut guard) = timing_history.write() {
                            *guard = Some(history);
                        }
                    }
                }
            }
        });
    }

    fn process_event(
        event: &BootstrapEvent,
        state: &Arc<RwLock<HashMap<String, ComponentState>>>,
        status: &Arc<RwLock<BootstrapStatus>>,
        detailed_cache: &Arc<Mutex<DetailedStatusCache>>,
    ) {
        match &event.payload {
            EventPayload::Started { total_components } => {
                if let Ok(mut guard) = status.write() {
                    guard.started_at = Some(event.timestamp);
                    guard.total_components = Some(*total_components);
                }
            }
            EventPayload::K3sDownloadStarted
            | EventPayload::K3sDownloadProgress { .. }
            | EventPayload::K3sDownloadCompleted
            | EventPayload::K3sInstallStarted
            | EventPayload::K3sInstallCompleted
            | EventPayload::K3sApiServerReady
            | EventPayload::K3sCoreDnsReady
            | EventPayload::K3sBootstrapCompleted => {
                // NOTE: Cluster init events are currently not surfaced in the TUI.
            }
            EventPayload::ComponentStarted { id } => {
                if let Ok(mut guard) = state.write() {
                    let entry = guard
                        .entry(id.clone())
                        .or_insert_with(|| ComponentState::new(id.clone()));
                    entry.mark_running(Instant::now());
                    entry.deferred_reason = None;
                }
                if let Ok(mut cache) = detailed_cache.lock() {
                    cache.invalidate(id);
                }
            }
            EventPayload::ComponentProgress {
                id,
                status: readiness,
                elapsed,
            } => {
                if let Ok(mut guard) = state.write() {
                    let entry = guard
                        .entry(id.clone())
                        .or_insert_with(|| ComponentState::new(id.clone()));
                    entry.status = ComponentStatus::Running;
                    entry.readiness = Some(readiness.clone());
                    entry.timing.update_elapsed(*elapsed);
                }
            }
            EventPayload::ComponentCompleted {
                id,
                duration,
                timing_breakdown,
            } => {
                if let Ok(mut guard) = state.write() {
                    let entry = guard
                        .entry(id.clone())
                        .or_insert_with(|| ComponentState::new(id.clone()));
                    entry.mark_completed(*duration);
                    entry.timing.render_duration = Some(timing_breakdown.render_duration);
                    entry.timing.apply_duration = Some(timing_breakdown.apply_duration);
                    entry.timing.wait_duration = Some(timing_breakdown.wait_duration);
                }
                if let Ok(mut cache) = detailed_cache.lock() {
                    cache.invalidate(id);
                }
            }
            EventPayload::ComponentFailed {
                id,
                duration,
                error,
            } => {
                if let Ok(mut guard) = state.write() {
                    let entry = guard
                        .entry(id.clone())
                        .or_insert_with(|| ComponentState::new(id.clone()));
                    entry.mark_failed(*duration);
                    entry.deferred_reason = Some(error.clone());
                }
                if let Ok(mut cache) = detailed_cache.lock() {
                    cache.invalidate(id);
                }
            }
            EventPayload::ComponentDeferred {
                id,
                reason,
                affected_dependents,
            } => {
                let reason_text = format!("{reason:?}");
                if let Ok(mut guard) = state.write() {
                    let entry = guard
                        .entry(id.clone())
                        .or_insert_with(|| ComponentState::new(id.clone()));
                    entry.mark_deferred(reason_text.clone());
                    for dep in affected_dependents {
                        let dep_entry = guard
                            .entry(dep.clone())
                            .or_insert_with(|| ComponentState::new(dep.clone()));
                        dep_entry.mark_deferred(format!("Dependency {id} deferred"));
                    }
                }
                if let Ok(mut cache) = detailed_cache.lock() {
                    cache.invalidate(id);
                    for dep in affected_dependents {
                        cache.invalidate(dep);
                    }
                }
            }
            EventPayload::Completed {
                total_duration,
                successful,
                failed,
                deferred,
            } => {
                if let Ok(mut guard) = status.write() {
                    guard.total_duration = Some(*total_duration);
                    guard.successful = *successful;
                    guard.failed = *failed;
                    guard.deferred = *deferred;
                }
            }
        }
    }

    async fn load_timing_history() -> Result<TimingHistory> {
        if let Ok(local) = LocalTimingStorage::new() {
            if let Ok(history) = local.load().await {
                return Ok(history);
            }
        }

        let client = K8sClient::new().await?;
        let cluster = ClusterTimingStorage::new(client.inner().clone());
        cluster.load().await
    }

    async fn fetch_access_urls(k8s: &K8sClient) -> Result<Vec<AccessUrlInfo>> {
        let api: kube::Api<Ingress> = kube::Api::all(k8s.inner().clone());
        let ingresses = api.list(&ListParams::default()).await?;
        let mut urls = Vec::new();

        for ingress in ingresses.items {
            let service = ingress.metadata.name.clone().unwrap_or_default();
            let status = ingress
                .status
                .as_ref()
                .and_then(|s| s.load_balancer.as_ref())
                .map(|lb| {
                    if lb.ingress.as_ref().map(|i| !i.is_empty()).unwrap_or(false) {
                        AccessStatus::Ready
                    } else {
                        AccessStatus::Pending
                    }
                })
                .unwrap_or(AccessStatus::Unknown);

            let Some(spec) = ingress.spec.as_ref() else {
                continue;
            };
            if let Some(rules) = spec.rules.as_ref() {
                for rule in rules {
                    if let Some(host) = rule.host.as_deref() {
                        let url = format!("https://{host}");
                        urls.push(AccessUrlInfo {
                            service: service.clone(),
                            url,
                            status,
                        });
                    }
                }
            }
        }

        Ok(urls)
    }

    async fn fetch_detailed_status_async(
        assembly: &Assembly,
        k8s: &K8sClient,
        component_id: &str,
    ) -> Result<DetailedStatus> {
        let step = assembly
            .steps
            .iter()
            .find(|step| step.id == component_id)
            .context("Component not found in assembly")?;

        for check in &step.checks {
            let Some((resource, checker)) = map_check_to_readiness(check, k8s) else {
                continue;
            };
            let detailed = checker.check_detailed(&resource).await?;
            if !detailed.pods.is_empty()
                || !detailed.conditions.is_empty()
                || !detailed.recent_events.is_empty()
            {
                return Ok(detailed);
            }
        }

        Ok(DetailedStatus::empty())
    }

    fn fetch_detailed_status_blocking(
        assembly: Arc<Assembly>,
        k8s: K8sClient,
        component_id: String,
    ) -> Result<DetailedStatus> {
        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new()?;
            rt.block_on(Self::fetch_detailed_status_async(
                &assembly,
                &k8s,
                &component_id,
            ))
        })
        .join()
        .map_err(|_| anyhow!("Detailed status worker thread panicked"))?
    }
}

impl BootstrapPort for BootstrapAdapter {
    fn component_states(&self) -> HashMap<String, ComponentState> {
        self.state
            .read()
            .map(|guard| guard.clone())
            .unwrap_or_default()
    }

    fn dependency_graph(&self) -> &Assembly {
        &self.assembly
    }

    fn timing_history(&self) -> Option<TimingHistory> {
        self.timing_history
            .read()
            .ok()
            .and_then(|guard| guard.clone())
    }

    fn bootstrap_status(&self) -> BootstrapStatus {
        self.status
            .read()
            .map(|guard| guard.clone())
            .unwrap_or_default()
    }

    fn access_urls(&self) -> Vec<AccessUrlInfo> {
        self.access_urls
            .read()
            .map(|guard| guard.clone())
            .unwrap_or_default()
    }

    fn send_command(&self, cmd: InteractiveCommand) -> Result<()> {
        self.command_tx
            .try_send(cmd)
            .context("Failed to send interactive command")
    }

    fn get_detailed_status(&self, component_id: &str) -> Result<DetailedStatus> {
        let mut cache = self
            .detailed_cache
            .lock()
            .map_err(|_| anyhow!("Detailed status cache lock poisoned"))?;
        if let Some(cached) = cache.get(component_id) {
            return Ok(cached);
        }

        let component_key = component_id.to_string();
        let detailed = match tokio::runtime::Handle::try_current() {
            Ok(handle) => {
                let assembly = Arc::clone(&self.assembly);
                let k8s = self.k8s.clone();
                let component_id = component_key.clone();
                if matches!(
                    handle.runtime_flavor(),
                    tokio::runtime::RuntimeFlavor::MultiThread
                ) {
                    tokio::task::block_in_place(|| {
                        Self::fetch_detailed_status_blocking(assembly, k8s, component_id)
                    })?
                } else {
                    Self::fetch_detailed_status_blocking(assembly, k8s, component_id)?
                }
            }
            Err(_) => {
                let rt = tokio::runtime::Runtime::new()?;
                rt.block_on(Self::fetch_detailed_status_async(
                    &self.assembly,
                    &self.k8s,
                    &component_key,
                ))?
            }
        };

        cache.insert(component_key, detailed.clone());
        Ok(detailed)
    }

    fn registry_specs(&self) -> HashMap<String, ModuleSpec> {
        let specs = primer::application::runtime::registry::get_all_specs();
        specs
            .into_iter()
            .map(|spec| (spec.name.to_string(), spec.clone()))
            .collect()
    }
}

struct DetailedStatusCache {
    data: HashMap<String, (DetailedStatus, Instant)>,
    ttl: Duration,
}

impl DetailedStatusCache {
    fn new(ttl: Duration) -> Self {
        Self {
            data: HashMap::new(),
            ttl,
        }
    }

    fn get(&mut self, component_id: &str) -> Option<DetailedStatus> {
        if let Some((status, timestamp)) = self.data.get(component_id) {
            if timestamp.elapsed() < self.ttl {
                return Some(status.clone());
            }
        }
        None
    }

    fn insert(&mut self, component_id: String, status: DetailedStatus) {
        self.data.insert(component_id, (status, Instant::now()));
    }

    fn invalidate(&mut self, component_id: &str) {
        self.data.remove(component_id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use primer::application::events::{DeferReason, EventPayload, TimingBreakdown};
    use primer::application::readiness::status::{
        BasicStatus, ReadinessPhase, ReadinessStatus,
    };

    fn make_event(payload: EventPayload) -> BootstrapEvent {
        BootstrapEvent {
            seq: 1,
            timestamp: Instant::now(),
            payload,
        }
    }

    fn setup() -> (
        Arc<RwLock<HashMap<String, ComponentState>>>,
        Arc<RwLock<BootstrapStatus>>,
        Arc<Mutex<DetailedStatusCache>>,
    ) {
        (
            Arc::new(RwLock::new(HashMap::new())),
            Arc::new(RwLock::new(BootstrapStatus::default())),
            Arc::new(Mutex::new(DetailedStatusCache::new(Duration::from_secs(5)))),
        )
    }

    #[test]
    fn test_component_lifecycle() {
        let (state, status, cache) = setup();
        let id = "test-comp".to_string();

        // 1. Started
        BootstrapAdapter::process_event(
            &make_event(EventPayload::ComponentStarted { id: id.clone() }),
            &state,
            &status,
            &cache,
        );

        {
            let guard = state.read().unwrap();
            let comp = guard.get(&id).expect("Component should exist");
            assert_eq!(comp.status, ComponentStatus::Running);
            assert!(comp.timing.started_at.is_some());
        }

        // 2. Progress
        BootstrapAdapter::process_event(
            &make_event(EventPayload::ComponentProgress {
                id: id.clone(),
                status: ReadinessStatus {
                    basic: BasicStatus {
                        phase: ReadinessPhase::Rendering,
                        summary: "loading".into(),
                        progress: Some(0.5),
                    },
                    detailed: None,
                },
                elapsed: Duration::from_secs(1),
            }),
            &state,
            &status,
            &cache,
        );

        {
            let guard = state.read().unwrap();
            let comp = guard.get(&id).unwrap();
            assert_eq!(comp.status, ComponentStatus::Running);
            assert_eq!(comp.readiness.as_ref().unwrap().basic.progress, Some(0.5));
        }

        // 3. Completed
        BootstrapAdapter::process_event(
            &make_event(EventPayload::ComponentCompleted {
                id: id.clone(),
                duration: Duration::from_secs(2),
                timing_breakdown: TimingBreakdown {
                    render_duration: Duration::from_millis(100),
                    apply_duration: Duration::from_millis(200),
                    wait_duration: Duration::from_millis(300),
                },
            }),
            &state,
            &status,
            &cache,
        );

        {
            let guard = state.read().unwrap();
            let comp = guard.get(&id).unwrap();
            assert_eq!(comp.status, ComponentStatus::Complete);
            assert_eq!(comp.timing.total_duration, Some(Duration::from_secs(2)));
        }
    }

    #[test]
    fn test_component_failure() {
        let (state, status, cache) = setup();
        let id = "fail-comp".to_string();

        BootstrapAdapter::process_event(
            &make_event(EventPayload::ComponentFailed {
                id: id.clone(),
                duration: Duration::from_secs(1),
                error: "Something went wrong".into(),
            }),
            &state,
            &status,
            &cache,
        );

        let guard = state.read().unwrap();
        let comp = guard.get(&id).expect("Component should exist");
        assert_eq!(comp.status, ComponentStatus::Failed);
        assert_eq!(comp.deferred_reason, Some("Something went wrong".into()));
    }

    #[test]
    fn test_component_deferred_cascade() {
        let (state, status, cache) = setup();
        let id = "upstream".to_string();
        let dep = "downstream".to_string();

        BootstrapAdapter::process_event(
            &make_event(EventPayload::ComponentDeferred {
                id: id.clone(),
                reason: DeferReason::DependencyFailed {
                    dependency: "other".into(),
                },
                affected_dependents: vec![dep.clone()],
            }),
            &state,
            &status,
            &cache,
        );

        let guard = state.read().unwrap();

        let upstream = guard.get(&id).unwrap();
        assert_eq!(upstream.status, ComponentStatus::Deferred);
        // Reason serialization check
        assert!(
            upstream
                .deferred_reason
                .as_ref()
                .unwrap()
                .contains("DependencyFailed")
        );

        let downstream = guard.get(&dep).unwrap();
        assert_eq!(downstream.status, ComponentStatus::Deferred);
        assert!(
            downstream
                .deferred_reason
                .as_ref()
                .unwrap()
                .contains("upstream deferred")
        );
    }

    #[test]
    fn test_cache_ttl() {
        let mut cache = DetailedStatusCache::new(Duration::from_millis(50));
        let id = "test".to_string();

        cache.insert(id.clone(), DetailedStatus::empty());
        assert!(cache.get(&id).is_some());

        std::thread::sleep(Duration::from_millis(60));
        assert!(cache.get(&id).is_none());
    }
}
