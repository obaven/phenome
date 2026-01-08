//! Reconcile Command Handler
//!
//! ## Responsibility
//! CLI entry point for the reconcile command. Routes to either:
//! - LifecycleManager (converge mode, one-shot bootstrap)
//! - Reconciler (watch mode, event-driven daemon)

use std::sync::Arc;
use tracing::{info, warn};

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
        "Starting Lifecycle Reconciler (config={}, watch={}, timing={}, cache={}, parallel={}, concurrency={}, force={})",
        args.assembly_path,
        args.watch,
        timing_enabled,
        args.cache,
        args.parallel,
        args.concurrency,
        args.force
    );

    // 1. Load Config
    bootstrappo::application::config::load()?;
    let config = bootstrappo::application::config::instance();

    // 1.5 Initialize infrastructure adapters
    let fs = Arc::new(
        bootstrappo::adapters::infrastructure::core::filesystem::RealFilesystemAdapter::new(),
    );
    let k8s = Arc::new(
        bootstrappo::adapters::infrastructure::kube::kubernetes::KubeRsAdapter::new().await?,
    );
    let helm = Arc::new(bootstrappo::adapters::infrastructure::helm::HelmBinaryAdapter::new());
    let cmd = Arc::new(
        bootstrappo::application::runtime::modules::io::command::CommandAdapter::new(),
    );

    // 2. Build Assembly
    let modules = bootstrappo::application::runtime::registry::get_all_modules(config.as_ref());
    let mut assembly = bootstrappo::application::composition::assembly::builder::AssemblyBuilder::new(
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

    if args.watch {
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

        // BSP-148: Pass force flag to context for fast-path skip bypass
        let mut manager = bootstrappo::application::lifecycle::LifecycleManager::new(
            config,
            bootstrappo::application::context::ModuleMode::Apply,
            fs.clone(),
            k8s.clone(),
            helm.clone(),
            cmd.clone(),
        )
        .with_plan(assembly);

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
