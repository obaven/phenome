use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use rotappo_domain::{Assembly, Event, HealthSnapshot};

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
}

impl PortSet {
    pub fn empty() -> Self {
        Self {
            assembly: Arc::new(NullAssemblyPort),
            health: Arc::new(NullHealthPort),
            logs: Arc::new(NullLogPort),
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
