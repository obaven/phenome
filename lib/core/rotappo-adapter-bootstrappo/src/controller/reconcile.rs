//! Reconcile Command Handler
//!
//! ## Responsibility
//! CLI entry point for the reconcile command. Routes to either:
//! - LifecycleManager (converge mode, one-shot bootstrap)
//! - Reconciler (watch mode, event-driven daemon)

use anyhow::Context;
use std::sync::Arc;
use tracing::{info, warn};

use bootstrappo::application::events::EventPayload;
use rotappo_domain::{Event, EventLevel};
use rotappo_ports::InMemoryLogPort;

#[derive(Debug, Clone)]
pub struct ReconcileArgs {
    pub assembly_path: String,
    pub watch: bool,
    pub overlay: Option<String>,
    pub timing: bool,
    pub timing_output: Option<String>,
    pub cache: bool,
    pub parallel: bool,
    pub concurrency: usize,
    pub force: bool,
    pub bootstrap_tui: bool,
}

/// Handles the `bootstrappo reconcile` command.
///
/// ## Behavior
/// - Watch mode: Uses event-driven Reconciler with cluster watchers
/// - Converge mode: Uses LifecycleManager for one-shot bootstrap
pub async fn reconcile(args: ReconcileArgs) -> anyhow::Result<()> {
    // BSP-145: Enable timing if output file is specified
    let timing_enabled = args.timing || args.timing_output.is_some();

    info!(
        "Starting Lifecycle Reconciler (config={}, watch={}, timing={}, cache={}, parallel={}, concurrency={}, force={}, bootstrap_tui={})",
        args.assembly_path,
        args.watch,
        timing_enabled,
        args.cache,
        args.parallel,
        args.concurrency,
        args.force,
        args.bootstrap_tui
    );

    // 1. Load Config
    bootstrappo::application::config::load()?;
    let config = bootstrappo::application::config::instance();

    // 1.5 Initialize infrastructure adapters
    let fs = Arc::new(
        bootstrappo::adapters::infrastructure::core::filesystem::RealFilesystemAdapter::new(),
    );
    let k8s_client =
        bootstrappo::adapters::infrastructure::kube::clients::k8s::K8sClient::new().await?;
    let k8s: Arc<dyn bootstrappo::ports::kubernetes::KubernetesPort> = Arc::new(k8s_client.clone());
    let helm = Arc::new(bootstrappo::adapters::infrastructure::helm::HelmBinaryAdapter::new());
    let cmd =
        Arc::new(bootstrappo::application::runtime::modules::io::command::CommandAdapter::new());

    // 2. Build Assembly
    let modules = bootstrappo::application::runtime::registry::get_all_modules(config.as_ref());
    let mut assembly =
        bootstrappo::application::composition::assembly::builder::AssemblyBuilder::new(
            config.as_ref().clone(),
        )
        .with_modules(modules)
        .build()?;

    // 3. Apply Overlay
    if let Some(ref o) = args.overlay {
        if let Some(ctx) = assembly.context.as_mut() {
            ctx.overlay = o.clone();
        }
    }

    if args.bootstrap_tui {
        if args.watch {
            warn!("--bootstrap-tui forces converge mode (watch disabled)");
        }

        let mode = bootstrappo::application::reconciler::ReconcileMode::Converge;

        let mut options = bootstrappo::application::reconciler::ReconcileOptions::default();
        if timing_enabled {
            options = options.with_timing();
        }
        if args.cache {
            options = options.with_caching();
        }
        if args.parallel {
            options = options.with_parallel(args.concurrency);
        }

        let discovery_client = kube::Client::try_default().await?;
        let discovery = Arc::new(
            bootstrappo::application::runtime::modules::runtime::k8s::cache::ClusterCache::new(
                discovery_client,
            ),
        );

        let event_bus = bootstrappo::application::events::EventBus::default();
        let log_port = InMemoryLogPort::default();
        let log_task = {
            let log_port = log_port.clone();
            let mut log_rx = event_bus.subscribe();
            tokio::spawn(async move {
                while let Ok(event) = log_rx.recv().await {
                    if let Some((level, message)) = format_bootstrap_log(&event.payload) {
                        log_port.push(Event::new(level, message));
                    }
                }
            })
        };
        let assembly_for_tui = assembly.clone();
        let mut reconciler = bootstrappo::application::reconciler::Reconciler::with_options(
            assembly,
            mode,
            options,
            fs.clone(),
            k8s.clone(),
            helm.clone(),
            cmd.clone(),
            discovery,
        )
        .await?;

        if let Some(o) = args.overlay.clone() {
            reconciler.override_overlay(o);
        }

        let (mut reconciler, command_tx) = reconciler.with_event_bus(event_bus.clone());

        let adapter = crate::bootstrap::BootstrapAdapter::new(
            event_bus,
            assembly_for_tui,
            command_tx,
            k8s_client.clone(),
        );
        let mut ports = rotappo_ports::PortSet::empty();
        ports.bootstrap = Arc::new(adapter);
        ports.logs = Arc::new(log_port);

        let tui_handle =
            tokio::task::spawn_blocking(move || rotappo_ui_tui::start_bootstrap(ports));
        let reconcile_handle = tokio::spawn(async move { reconciler.run().await });

        tui_handle.await.context("Bootstrap TUI task failed")??;
        reconcile_handle.await.context("Reconciler task failed")??;
        let _ = log_task.await;

        info!("Bootstrap TUI session completed.");
    } else if args.watch {
        // Watch mode: Use the event-driven Reconciler
        info!("Running in Watch mode with event-driven Reconciler");

        let mode = bootstrappo::application::reconciler::ReconcileMode::Watch;

        // Build options with enabled features
        let mut options = bootstrappo::application::reconciler::ReconcileOptions::default();
        if timing_enabled {
            options = options.with_timing();
        }
        if args.cache {
            options = options.with_caching();
        }
        if args.parallel {
            options = options.with_parallel(args.concurrency);
        }

        let discovery_client = kube::Client::try_default().await?;
        let discovery = Arc::new(
            bootstrappo::application::runtime::modules::runtime::k8s::cache::ClusterCache::new(
                discovery_client,
            ),
        );
        let mut reconciler = bootstrappo::application::reconciler::Reconciler::with_options(
            assembly,
            mode,
            options,
            fs.clone(),
            k8s.clone(),
            helm.clone(),
            cmd.clone(),
            discovery,
        )
        .await?;

        // Apply overlay if specified
        if let Some(o) = args.overlay {
            reconciler.override_overlay(o);
        }

        reconciler.run().await?;
        info!("Reconciler watch loop exited.");
    } else {
        // Converge mode: Use LifecycleManager for one-shot bootstrap
        info!("Running in Converge mode with LifecycleManager");

        // BSP-227: Create native K8sClient for namespace and manifest operations
        let k8s_client = Some(k8s_client.clone());

        let discovery_client = kube::Client::try_default().await?;
        let discovery = Arc::new(
            bootstrappo::application::runtime::modules::runtime::k8s::cache::ClusterCache::new(
                discovery_client,
            ),
        );

        // BSP-148: Pass force flag to context for fast-path skip bypass
        let mut manager = bootstrappo::application::lifecycle::LifecycleManager::new(
            config,
            bootstrappo::application::context::ModuleMode::Apply,
            fs.clone(),
            k8s.clone(),
            helm.clone(),
            cmd.clone(),
            discovery,
        )
        .with_plan(assembly);

        // BSP-227: Set native K8sClient if available
        if let Some(client) = k8s_client {
            manager = manager.with_k8s_client(client);
        }

        // Set force mode on the context
        manager.context.force = args.force;

        if args.force {
            info!("Force mode enabled: bypassing fast-path convergence checks");
        }

        // BSP-153: Enable cache for Helm charts if --cache flag is set
        if args.cache {
            match bootstrappo::adapters::cache::CacheManager::new() {
                Ok(cache_manager) => {
                    info!("Artifact cache enabled");
                    manager = manager.with_cache(std::sync::Arc::new(cache_manager));
                }
                Err(e) => {
                    warn!("Failed to initialize cache, continuing without: {}", e);
                }
            }
        }

        // BSP-145: Enable timing in converge mode
        if timing_enabled {
            info!("Timing instrumentation enabled (converge mode)");
            bootstrappo::application::bootstrap::bootstrap_with_timing(
                &manager,
                args.timing_output.as_deref(),
            )
            .await?;
        } else {
            if args.parallel {
                info!("Note: --parallel is currently only supported in --watch mode");
            }
            manager.bootstrap().await?;
        }

        info!("Lifecycle Bootstrap completed successfully.");
    }

    Ok(())
}

fn format_bootstrap_log(payload: &EventPayload) -> Option<(EventLevel, String)> {
    match payload {
        EventPayload::Started { total_components } => Some((
            EventLevel::Info,
            format!("bootstrap started (components: {total_components})"),
        )),
        EventPayload::ComponentStarted { id } => {
            Some((EventLevel::Info, format!("component {id} started")))
        }
        EventPayload::ComponentProgress { .. } => None,
        EventPayload::ComponentCompleted { id, duration, .. } => Some((
            EventLevel::Info,
            format!("component {id} completed in {}s", duration.as_secs()),
        )),
        EventPayload::ComponentFailed { id, error, .. } => {
            Some((EventLevel::Error, format!("component {id} failed: {error}")))
        }
        EventPayload::ComponentDeferred {
            id,
            reason,
            affected_dependents,
        } => {
            let dependents = if affected_dependents.is_empty() {
                "none".to_string()
            } else {
                affected_dependents.join(", ")
            };
            Some((
                EventLevel::Warn,
                format!("component {id} deferred ({reason:?}); dependents: {dependents}"),
            ))
        }
        EventPayload::Completed {
            total_duration,
            successful,
            failed,
            deferred,
        } => {
            let level = if *failed > 0 {
                EventLevel::Warn
            } else {
                EventLevel::Info
            };
            Some((
                level,
                format!(
                    "bootstrap completed in {}s (ok: {successful}, failed: {failed}, deferred: {deferred})",
                    total_duration.as_secs()
                ),
            ))
        }
        EventPayload::K3sDownloadStarted => Some((EventLevel::Info, "k3s download started".into())),
        EventPayload::K3sDownloadProgress { percent } => Some((
            EventLevel::Info,
            format!("k3s download {:.0}%", percent * 100.0),
        )),
        EventPayload::K3sDownloadCompleted => {
            Some((EventLevel::Info, "k3s download completed".into()))
        }
        EventPayload::K3sInstallStarted => Some((EventLevel::Info, "k3s install started".into())),
        EventPayload::K3sInstallCompleted => {
            Some((EventLevel::Info, "k3s install completed".into()))
        }
        EventPayload::K3sApiServerReady => Some((EventLevel::Info, "k3s API server ready".into())),
        EventPayload::K3sCoreDnsReady => Some((EventLevel::Info, "k3s CoreDNS ready".into())),
        EventPayload::K3sBootstrapCompleted => {
            Some((EventLevel::Info, "k3s bootstrap completed".into()))
        }
    }
}
