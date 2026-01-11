use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use rotappo_domain::{Assembly, Event, HealthSnapshot};

mod bootstrap;

pub use bootstrap::{
    AccessStatus, AccessUrlInfo, BootstrapPort, BootstrapStatus, ComponentState, ComponentStatus,
    ComponentTiming, InteractiveCommand,
};

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
}

impl PortSet {
    pub fn empty() -> Self {
        Self {
            assembly: Arc::new(NullAssemblyPort),
            health: Arc::new(NullHealthPort),
            logs: Arc::new(NullLogPort),
            bootstrap: Arc::new(NullBootstrapPort),
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

    fn dependency_graph(&self) -> &bootstrappo::domain::models::assembly::Assembly {
        static EMPTY: std::sync::OnceLock<bootstrappo::domain::models::assembly::Assembly> =
            std::sync::OnceLock::new();
        EMPTY.get_or_init(bootstrappo::domain::models::assembly::Assembly::default)
    }

    fn timing_history(&self) -> Option<bootstrappo::application::timing::TimingHistory> {
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
        _cmd: bootstrappo::application::events::InteractiveCommand,
    ) -> anyhow::Result<()> {
        Ok(())
    }

    fn get_detailed_status(
        &self,
        _component_id: &str,
    ) -> anyhow::Result<bootstrappo::application::readiness::DetailedStatus> {
        Ok(bootstrappo::application::readiness::DetailedStatus::empty())
    }
}
