use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};

use bootstrappo::ops::drivers::HealthStatus;
use bootstrappo::ops::k8s::cache::ClusterCache;
use bootstrappo::ops::reconciler::plan::Plan;

use crate::runtime::Event;

pub trait PlanPort: Send + Sync {
    fn plan(&self) -> Option<Plan>;
    fn plan_error(&self) -> Option<String>;
}

pub trait HealthPort: Send + Sync {
    fn health(&self) -> HashMap<String, HealthStatus>;
    fn last_error(&self) -> Option<String>;
}

pub trait CachePort: Send + Sync {
    fn cache(&self) -> Option<ClusterCache>;
}

pub trait LogPort: Send + Sync {
    fn drain_events(&self) -> Vec<Event>;
}

#[derive(Clone)]
pub struct PortSet {
    pub plan: Arc<dyn PlanPort>,
    pub health: Arc<dyn HealthPort>,
    pub cache: Arc<dyn CachePort>,
    pub logs: Arc<dyn LogPort>,
}

impl PortSet {
    pub fn empty() -> Self {
        Self {
            plan: Arc::new(NullPlanPort),
            health: Arc::new(NullHealthPort),
            cache: Arc::new(NullCachePort),
            logs: Arc::new(NullLogPort),
        }
    }
}

#[derive(Clone, Default)]
struct NullPlanPort;

impl PlanPort for NullPlanPort {
    fn plan(&self) -> Option<Plan> {
        None
    }

    fn plan_error(&self) -> Option<String> {
        None
    }
}

#[derive(Clone, Default)]
struct NullHealthPort;

impl HealthPort for NullHealthPort {
    fn health(&self) -> HashMap<String, HealthStatus> {
        HashMap::new()
    }

    fn last_error(&self) -> Option<String> {
        None
    }
}

#[derive(Clone, Default)]
struct NullCachePort;

impl CachePort for NullCachePort {
    fn cache(&self) -> Option<ClusterCache> {
        None
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
