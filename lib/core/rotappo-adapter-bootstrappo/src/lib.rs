pub mod controller;
mod health;
mod mapping;
mod assembly;

use anyhow::{Context, Result};
use std::path::PathBuf;
use std::sync::Arc;

use rotappo_ports::{LogPort, PortSet};
use rotappo_domain::Event;

pub use health::LiveStatus;

pub struct BootstrappoBackend {
    pub config: Arc<bootstrappo_api::contract::config::Config>,
    pub config_path: PathBuf,
    pub assembly_path: PathBuf,
    pub assembly: Option<rotappo_domain::Assembly>,
    pub assembly_error: Option<String>,
    pub live_status: Option<LiveStatus>,
    ports: PortSet,
}

impl BootstrappoBackend {
    pub fn from_env() -> Result<Self> {
        let config_path = std::env::var("BOOTSTRAPPO_CONFIG_PATH")
            .map(PathBuf::from)
            .ok();
        let assembly_path = std::env::var("BOOTSTRAPPO_ASSEMBLY_PATH")
            .map(PathBuf::from)
            .ok();
        Self::from_paths(config_path, assembly_path)
    }

    pub fn from_paths(
        config_path: Option<PathBuf>,
        assembly_path: Option<PathBuf>,
    ) -> Result<Self> {
        let config_path = config_path.unwrap_or_else(|| {
            PathBuf::from("../bootstrappo/data/configs/bootstrap-config.yaml")
        });
        let config =
            bootstrappo::application::config::load_from_file(&config_path).with_context(|| {
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
        let health_port = health::BootstrappoHealthPort::new(live_status.clone());
        let ports = PortSet {
            assembly: Arc::new(assembly_port),
            health: Arc::new(health_port),
            logs: Arc::new(BootstrappoLogPort),
        };

        Ok(Self {
            config,
            config_path,
            assembly_path,
            assembly,
            assembly_error,
            live_status,
            ports,
        })
    }

    pub fn ports(&self) -> PortSet {
        self.ports.clone()
    }
}

#[derive(Clone, Copy)]
struct BootstrappoLogPort;

impl LogPort for BootstrappoLogPort {
    fn drain_events(&self) -> Vec<Event> {
        Vec::new()
    }
}
