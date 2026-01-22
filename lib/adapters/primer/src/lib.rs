pub mod controller;
mod runtime;

pub use runtime::{assembly, bootstrap, health, mapping};

use anyhow::{Context, Result};
use std::path::PathBuf;
use std::sync::Arc;

use primer::adapters::infrastructure::kube::clients::k8s::K8sClient;
use primer::application::events::{EventBus, InteractiveCommand};
use phenome_domain::Event;
use phenome_ports::{LogPort, PortSet};
use tokio::sync::mpsc;

pub use runtime::bootstrap::BootstrapAdapter;
pub use runtime::health::LiveStatus;

pub struct BootstrappoBackend {
    pub config: Arc<bootstrappo_api::contract::config::Config>,
    pub config_path: PathBuf,
    pub assembly_path: PathBuf,
    pub assembly: Option<phenome_domain::Assembly>,
    pub assembly_error: Option<String>,
    pub live_status: Option<LiveStatus>,
    ports: PortSet,
    bootstrap_event_bus: EventBus,
    bootstrap_command_tx: mpsc::Sender<InteractiveCommand>,
    bootstrap_command_rx: Option<mpsc::Receiver<InteractiveCommand>>,
    bootstrap_runtime: Option<tokio::runtime::Runtime>,
}

impl BootstrappoBackend {
    pub fn from_env() -> Result<Self> {
        let config_path = std::env::var("PRIMER_CONFIG_PATH")
            .map(PathBuf::from)
            .ok();
        let assembly_path = std::env::var("PRIMER_ASSEMBLY_PATH")
            .map(PathBuf::from)
            .ok();
        Self::from_paths(config_path, assembly_path)
    }

    pub fn from_paths(
        config_path: Option<PathBuf>,
        assembly_path: Option<PathBuf>,
    ) -> Result<Self> {
        let bootstrap_event_bus = EventBus::default();
        let (bootstrap_command_tx, bootstrap_command_rx) = mpsc::channel(100);
        Self::build_with_bootstrap(
            config_path,
            assembly_path,
            bootstrap_event_bus,
            bootstrap_command_tx,
            Some(bootstrap_command_rx),
        )
    }

    pub fn from_paths_with_bootstrap(
        config_path: Option<PathBuf>,
        assembly_path: Option<PathBuf>,
        bootstrap_event_bus: EventBus,
        bootstrap_command_tx: mpsc::Sender<InteractiveCommand>,
    ) -> Result<Self> {
        Self::build_with_bootstrap(
            config_path,
            assembly_path,
            bootstrap_event_bus,
            bootstrap_command_tx,
            None,
        )
    }

    fn build_with_bootstrap(
        config_path: Option<PathBuf>,
        assembly_path: Option<PathBuf>,
        bootstrap_event_bus: EventBus,
        bootstrap_command_tx: mpsc::Sender<InteractiveCommand>,
        bootstrap_command_rx: Option<mpsc::Receiver<InteractiveCommand>>,
    ) -> Result<Self> {
        let config_path = config_path
            .unwrap_or_else(|| PathBuf::from("../primer/data/configs/bootstrap-config.yaml"));
        let config =
            primer::application::config::load_from_file(&config_path).with_context(|| {
                format!(
                    "Failed to load Bootstrappo config at {}",
                    config_path.display()
                )
            })?;

        let assembly_path = assembly_path.unwrap_or_else(|| config_path.clone());
        let config = Arc::new(config);
        let live_status = Some(LiveStatus::spawn(Arc::clone(&config)));
        let assembly_port =
            assembly::BootstrappoAssemblyPort::load(live_status.clone(), Arc::clone(&config));
        let assembly = assembly_port.assembly();
        let assembly_error = assembly_port.assembly_error();
        let bootstrap_assembly = assembly_port.bootstrappo_assembly().unwrap_or_default();
        let health_port = health::BootstrappoHealthPort::new(live_status.clone());
        let mut ports = PortSet::empty();
        ports.assembly = Arc::new(assembly_port);
        ports.health = Arc::new(health_port);
        ports.logs = Arc::new(BootstrappoLogPort);
        let (bootstrap_runtime, handle) = match tokio::runtime::Handle::try_current() {
            Ok(handle) => (None, handle),
            Err(_) => {
                let runtime = tokio::runtime::Builder::new_multi_thread()
                    .enable_all()
                    .build()?;
                let handle = runtime.handle().clone();
                (Some(runtime), handle)
            }
        };
        let k8s = handle.block_on(K8sClient::new())?;
        let _guard = handle.enter();
        let bootstrap_adapter = BootstrapAdapter::new(
            bootstrap_event_bus.clone(),
            bootstrap_assembly,
            bootstrap_command_tx.clone(),
            k8s,
        );
        ports.bootstrap = Arc::new(bootstrap_adapter);

        Ok(Self {
            config,
            config_path,
            assembly_path,
            assembly,
            assembly_error,
            live_status,
            ports,
            bootstrap_event_bus,
            bootstrap_command_tx,
            bootstrap_command_rx,
            bootstrap_runtime,
        })
    }

    pub fn ports(&self) -> PortSet {
        self.ports.clone()
    }

    pub fn bootstrap_event_bus(&self) -> &EventBus {
        &self.bootstrap_event_bus
    }

    pub fn bootstrap_command_sender(&self) -> &mpsc::Sender<InteractiveCommand> {
        &self.bootstrap_command_tx
    }

    pub fn take_bootstrap_command_receiver(
        &mut self,
    ) -> Option<mpsc::Receiver<InteractiveCommand>> {
        self.bootstrap_command_rx.take()
    }

    pub fn bootstrap_runtime(&self) -> Option<&tokio::runtime::Runtime> {
        self.bootstrap_runtime.as_ref()
    }
}

#[derive(Clone, Copy)]
struct BootstrappoLogPort;

impl LogPort for BootstrappoLogPort {
    fn drain_events(&self) -> Vec<Event> {
        Vec::new()
    }
}
