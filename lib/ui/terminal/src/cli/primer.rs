use clap::{Args, Parser, Subcommand};
use tracing::info;

use phenome_adapter_primer::controller as adapter_controller;

#[derive(Parser)]
#[command(name = "primer")]
#[command(about = "Primer - Declarative Kubernetes Platform Bootstrap")]
#[command(long_about = r#"
Primer - Declarative Kubernetes Platform Bootstrap

Primary workflow:
  1. Edit ../primer/data/configs/bootstrap-config.yaml
  2. Run: primer reconcile
  3. Cluster converges to desired state

For more info: primer <command> --help
"#)]
pub struct Cli {
    /// Path to the Primer config file
    #[arg(
        long = "config",
        alias = "config-path",
        env = "PRIMER_CONFIG_PATH",
        global = true,
        default_value = "../primer/data/configs/bootstrap-config.yaml"
    )]
    pub config_path: String,

    /// Path to the GitOps directory
    #[arg(
        long,
        env = "PRIMER_GITOPS_DIR",
        global = true,
        default_value = "/home/jdean/primer/data/output/app/exoskeleton"
    )]
    pub gitops_dir: String,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Assembly management utilities
    Assembly {
        #[command(subcommand)]
        assembly: AssemblyAction,
    },

    /// Show checks and signals observed in the cluster
    Status,

    /// Compare enabled modules vs satisfied signals
    Diff,

    /// Explain a module (requirements, checks, engine, hooks)
    Explain {
        /// Module name to explain
        module: String,
    },

    /// Print or export the module catalog
    Catalog {
        /// Write catalog output to a file instead of stdout
        #[arg(long)]
        output: Option<String>,
    },

    /// Manually trigger rotations (ingress, tls, dns, policy)
    Rotate {
        /// Rotation type to trigger
        #[arg(value_parser = ["ingress", "tls", "dns", "policy", "all"])]
        rotation: String,

        /// Path to the bootstrap config YAML
        #[arg(
            long = "assembly",
            default_value = "../primer/data/configs/bootstrap-config.yaml"
        )]
        assembly: String,

        /// Preview changes without executing
        #[arg(long, default_value_t = false)]
        dry_run: bool,
    },

    /// Run the event-driven reconciler
    Reconcile {
        /// Path to the bootstrap config YAML
        #[arg(
            long = "assembly",
            default_value = "../primer/data/configs/bootstrap-config.yaml"
        )]
        assembly: String,

        /// Run in daemon mode (watch for changes)
        #[arg(long, default_value_t = false)]
        watch: bool,

        /// Override the overlay context (prod|dev|test)
        #[arg(long)]
        overlay: Option<String>,

        /// Enable per-step timing instrumentation and hotspot report
        #[arg(long, default_value_t = false)]
        timing: bool,

        /// BSP-145: Output timing data to JSON file
        #[arg(long)]
        timing_output: Option<String>,

        /// Enable step-level render caching (skip unchanged steps)
        #[arg(long, default_value_t = false)]
        cache: bool,

        /// Enable parallel execution of independent steps
        #[arg(long, default_value_t = false)]
        parallel: bool,

        /// Maximum concurrent steps when --parallel is enabled
        #[arg(long, default_value_t = 4)]
        concurrency: usize,

        /// BSP-148: Force re-reconcile all components, bypassing fast-path skip
        #[arg(long, default_value_t = false)]
        force: bool,

        /// Launch the bootstrap TUI dashboard
        #[arg(long = "bootstrap-tui", default_value_t = false)]
        bootstrap_tui: bool,

        /// Enable interactive controls (pause/resume/skip)
        #[arg(long, default_value_t = false)]
        interactive: bool,
    },

    /// Cluster lifecycle utilities
    Cluster {
        #[command(subcommand)]
        action: ClusterAction,
    },

    /// Aggressively delete all resources from the cluster
    Nuke {
        /// Path to the bootstrap config YAML
        #[arg(
            long = "assembly",
            default_value = "../primer/data/configs/bootstrap-config.yaml"
        )]
        assembly: String,

        /// Skip confirmation prompt
        #[arg(long, default_value_t = false)]
        aggressive: bool,

        /// Preview deletion order without executing
        #[arg(long, default_value_t = false)]
        dry_run: bool,
    },

    /// Debug utilities for inspecting registry and assembly
    Debug {
        #[command(subcommand)]
        action: DebugAction,
    },

    /// BSP-146: Manage artifact cache
    Cache {
        #[command(subcommand)]
        action: CacheAction,
    },

    /// Generates configuration from system discovery
    Generate(GenerateArgs),

    /// Visualization tools
    Visualize(VisualizeArgs),
}

#[derive(Subcommand)]
pub enum AssemblyAction {
    /// Validate a bootstrap config file
    Validate {
        /// Path to the bootstrap config YAML
        #[arg(default_value = "../primer/data/configs/bootstrap-config.yaml")]
        path: String,
    },

    /// Generate a visualization of the assembly
    Visualize {
        /// Path to the bootstrap config YAML
        #[arg(default_value = "../primer/data/configs/bootstrap-config.yaml")]
        path: String,

        /// View type (full, storage, network, dependencies)
        #[arg(long, default_value = "full")]
        view: String,

        /// Output format (svg, dot, png)
        #[arg(long, default_value = "svg")]
        format: String,

        /// Layout engine (dot, neato, fdp, circo)
        #[arg(long, default_value = "dot")]
        layout: String,

        /// Output file path (optional, defaults to stdout for DOT/SVG bytes)
        #[arg(short, long)]
        output: Option<String>,
    },
}

#[derive(Subcommand)]
pub enum ClusterAction {
    /// Initialize k3s cluster
    Init {
        /// Skip upgrade prompt (use existing cluster as-is)
        #[arg(long, default_value_t = false)]
        skip_upgrade: bool,

        /// Force reinstall even if cluster exists
        #[arg(long, default_value_t = false)]
        force: bool,
    },
}

#[derive(Subcommand)]
pub enum DebugAction {
    /// List all registered modules
    Registry,

    /// Print the execution order of the assembly
    AssemblyOrder {
        /// Path to the bootstrap config YAML
        #[arg(
            long = "assembly",
            default_value = "../primer/data/configs/bootstrap-config.yaml"
        )]
        assembly: String,
    },
}

#[derive(Subcommand)]
pub enum CacheAction {
    /// Show cache statistics
    Status,
    /// Clear all cached data
    Purge {
        /// Skip confirmation prompt
        #[arg(long, default_value_t = false)]
        force: bool,
    },
}

#[derive(Debug, Args)]
pub struct GenerateArgs {
    #[clap(subcommand)]
    pub command: GenerateCommands,
}

#[derive(Debug, Subcommand)]
pub enum GenerateCommands {
    /// Generate storage configuration by scanning system devices
    Storage(StorageArgs),
}

#[derive(Debug, Args, Clone)]
pub struct StorageArgs {
    /// Minimum size in GB to include
    #[clap(long)]
    pub min_size: Option<u64>,
}

#[derive(Debug, Args)]
pub struct VisualizeArgs {
    #[clap(subcommand)]
    pub command: VisualizeCommands,

    /// Output format (svg, dot, png)
    #[arg(long, default_value = "svg", global = true)]
    pub format: String,

    /// Layout engine (dot, neato, fdp, circo)
    #[arg(long, default_value = "dot", global = true)]
    pub layout: String,

    /// Output file path (optional, defaults to stdout for DOT/SVG bytes)
    #[arg(short, long, global = true)]
    pub output: Option<String>,
}

#[derive(Debug, Subcommand)]
pub enum VisualizeCommands {
    /// Visualize system generation results
    Generate(GenerateVisualizeArgs),
}

#[derive(Debug, Args)]
pub struct GenerateVisualizeArgs {
    #[clap(subcommand)]
    pub command: GenerateSubCommands,
}

#[derive(Debug, Subcommand)]
pub enum GenerateSubCommands {
    /// Visualize discovered storage devices
    Storage(StorageArgs),
}

pub async fn run() -> anyhow::Result<()> {
    dotenv::dotenv().ok();
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .init();

    let cli = Cli::parse();
    // Safe at CLI startup before any threads touch the environment.
    unsafe {
        std::env::set_var("PRIMER_CONFIG_PATH", &cli.config_path);
    }
    let summary = adapter_controller::load_config_summary();
    if let Some(error) = &summary.load_error {
        tracing::warn!(
            "Failed to load configuration from {}: {}. using defaults.",
            cli.config_path,
            error
        );
    }

    info!("Primer initializing...");
    info!("Using Config Path: {}", cli.config_path);
    info!("Using GitOps Directory: {}", cli.gitops_dir);
    if summary.load_error.is_none() {
        info!("Using Host Domain: {}", summary.host_domain);
        for pool in summary.metallb_pools {
            info!("Using MetalLB Pool [{}]: {}", pool.name, pool.ip_range);
        }
    }

    dispatch(cli.command, cli.gitops_dir).await
}

async fn dispatch(command: Commands, gitops_dir: String) -> anyhow::Result<()> {
    match command {
        Commands::Assembly { assembly } => match assembly {
            AssemblyAction::Validate { path } => adapter_controller::assembly::validate(path).await,
            AssemblyAction::Visualize {
                path,
                view,
                format,
                layout,
                output,
            } => {
                adapter_controller::assembly::visualize(
                    adapter_controller::assembly::AssemblyVisualizeArgs {
                        path,
                        view,
                        format,
                        layout,
                        output,
                    },
                )
                .await
            }
        },
        Commands::Status => adapter_controller::status::status().await,
        Commands::Diff => adapter_controller::diff::diff().await,
        Commands::Explain { module } => adapter_controller::explain::explain(module).await,
        Commands::Catalog { output } => adapter_controller::catalog::catalog(output).await,
        Commands::Rotate {
            rotation,
            assembly,
            dry_run,
        } => adapter_controller::rotate::rotate(rotation, assembly, gitops_dir, dry_run).await,
        Commands::Reconcile {
            assembly,
            watch,
            overlay,
            timing,
            timing_output,
            cache,
            parallel,
            concurrency,
            force,
            bootstrap_tui,
            interactive,
        } => {
            adapter_controller::reconcile::reconcile(adapter_controller::reconcile::ReconcileArgs {
                assembly_path: assembly,
                watch,
                overlay,
                timing,
                timing_output,
                cache,
                parallel,
                concurrency,
                force,
                bootstrap_tui,
                interactive,
            })
            .await
        }
        Commands::Cluster { action } => match action {
            ClusterAction::Init {
                skip_upgrade,
                force,
            } => adapter_controller::cluster::init(skip_upgrade, force).await,
        },
        Commands::Nuke {
            assembly,
            aggressive,
            dry_run,
        } => adapter_controller::nuke::nuke(assembly, aggressive, dry_run).await,
        Commands::Debug { action } => match action {
            DebugAction::Registry => adapter_controller::debug::registry().await,
            DebugAction::AssemblyOrder { assembly } => {
                adapter_controller::debug::assembly_order(assembly).await
            }
        },
        Commands::Cache { action } => match action {
            CacheAction::Status => adapter_controller::cache::status().await,
            CacheAction::Purge { force } => adapter_controller::cache::purge(force).await,
        },
        Commands::Generate(args) => match args.command {
            GenerateCommands::Storage(storage) => {
                adapter_controller::generate::storage(adapter_controller::generate::StorageArgs {
                    min_size: storage.min_size,
                })
                .await
            }
        },
        Commands::Visualize(args) => match args.command {
            VisualizeCommands::Generate(gen_args) => match gen_args.command {
                GenerateSubCommands::Storage(storage) => {
                    adapter_controller::visualize::generate_storage(
                        adapter_controller::generate::StorageArgs {
                            min_size: storage.min_size,
                        },
                        args.format,
                        args.layout,
                        args.output,
                    )
                    .await
                }
            },
        },
    }
}
