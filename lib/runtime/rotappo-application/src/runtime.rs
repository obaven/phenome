use anyhow::{Result, anyhow};

use phenome_domain::{ActionId, ActionRegistry, ActionSafety};
use phenome_domain::{
    ActionStatus, Assembly, AssemblyStep, AssemblyStepDef, AssemblyStepStatus, Snapshot,
};
use phenome_domain::{Event, EventBus, EventLevel};
use phenome_ports::PortSet;

pub struct Runtime {
    registry: ActionRegistry,
    snapshot: Snapshot,
    events: EventBus,
    refresh_count: u64,
    assembly: Option<Assembly>,
    ports: PortSet,
}

impl Runtime {
    pub fn new(registry: ActionRegistry) -> Self {
        Self::new_with_ports(registry, PortSet::empty())
    }

    pub fn new_with_ports(registry: ActionRegistry, ports: PortSet) -> Self {
        let mut events = EventBus::default();
        events.push(Event::new(EventLevel::Info, "Runtime initialized"));
        let assembly = ports.assembly.assembly();
        let snapshot = match assembly.as_ref() {
            Some(assembly) => Self::snapshot_from_assembly(assembly),
            None => Snapshot::new_default(),
        };

        let mut runtime = Self {
            registry,
            snapshot,
            events,
            refresh_count: 0,
            assembly,
            ports,
        };
        runtime.drain_port_events();
        runtime.snapshot.update_assembly_summary_from_steps();
        runtime
    }

    pub fn registry(&self) -> &ActionRegistry {
        &self.registry
    }

    pub fn snapshot(&self) -> &Snapshot {
        &self.snapshot
    }

    pub fn events(&self) -> &EventBus {
        &self.events
    }

    pub fn events_mut(&mut self) -> &mut EventBus {
        &mut self.events
    }

    pub fn refresh_snapshot(&mut self) {
        self.refresh_count = self.refresh_count.saturating_add(1);
        self.drain_port_events();
        if !self.snapshot.assembly_steps.is_empty() {
            self.update_action_statuses();
            self.sync_capabilities_from_steps();
        }
        self.snapshot.touch();
    }

    fn drain_port_events(&mut self) {
        for event in self.ports.logs.drain_events() {
            self.events.push(event);
        }
    }

    pub fn trigger_action(&mut self, action_id: ActionId) -> Result<()> {
        let action_def = self
            .registry
            .get(action_id)
            .ok_or_else(|| anyhow!("Unknown action: {action_id}"))?;

        if action_def.safety == ActionSafety::Destructive {
            self.events.push(Event::new(
                EventLevel::Warn,
                format!("Destructive action queued: {}", action_def.label),
            ));
        }

        self.snapshot.mark_action(action_id, ActionStatus::Running);
        self.events.push(Event::new(
            EventLevel::Info,
            format!("Started action: {}", action_def.label),
        ));

        self.snapshot
            .mark_action(action_id, ActionStatus::Succeeded);
        self.events.push(Event::new(
            EventLevel::Info,
            format!("Completed action: {}", action_def.label),
        ));

        self.snapshot.touch();
        Ok(())
    }

    fn snapshot_from_assembly(assembly: &Assembly) -> Snapshot {
        let mut snapshot = Snapshot::new_default();
        snapshot.assembly_steps = assembly.steps.iter().map(assembly_step_from_def).collect();

        snapshot.capabilities = assembly
            .steps
            .iter()
            .flat_map(|step| step.provides.iter().cloned())
            .collect::<std::collections::BTreeSet<_>>()
            .into_iter()
            .map(|name| phenome_domain::Capability {
                name,
                status: phenome_domain::CapabilityStatus::Degraded,
            })
            .collect();

        snapshot.update_assembly_summary_from_steps();
        snapshot
    }

    fn update_action_statuses(&mut self) {
        let assembly = match &self.assembly {
            Some(assembly) => assembly,
            None => {
                self.snapshot.update_assembly_summary_from_steps();
                return;
            }
        };

        let health_snapshot = self.ports.health.snapshot();
        let readiness = self.ports.assembly.step_readiness();
        let step_map: std::collections::HashMap<_, _> = assembly
            .steps
            .iter()
            .map(|step| (step.id.as_str(), step))
            .collect();

        let statuses: Vec<AssemblyStepStatus> = self
            .snapshot
            .assembly_steps
            .iter()
            .map(|step| {
                let blocked = step.depends_on.iter().any(|dep| {
                    self.snapshot.assembly_steps.iter().any(|other| {
                        other.id == *dep && other.status != AssemblyStepStatus::Succeeded
                    })
                });

                let mut status = if blocked {
                    AssemblyStepStatus::Blocked
                } else {
                    AssemblyStepStatus::Running
                };

                if let Some(def) = step_map.get(step.id.as_str()) {
                    let is_ready = readiness.get(step.id.as_str()).copied().unwrap_or(false);
                    if is_ready || (!def.has_gates && !blocked) {
                        status = AssemblyStepStatus::Succeeded;
                    }
                }

                if let Some(health) = health_snapshot.health.get(&step.id) {
                    status = match health {
                        phenome_domain::ComponentHealthStatus::Healthy => status,
                        phenome_domain::ComponentHealthStatus::Degraded(_) => {
                            AssemblyStepStatus::Running
                        }
                        phenome_domain::ComponentHealthStatus::Unhealthy(_) => {
                            AssemblyStepStatus::Failed
                        }
                    };
                }

                status
            })
            .collect();

        for (step, status) in self.snapshot.assembly_steps.iter_mut().zip(statuses) {
            step.status = status;
        }
        self.snapshot.update_assembly_summary_from_steps();
    }

    fn sync_capabilities_from_steps(&mut self) {
        let completed: std::collections::BTreeSet<String> = self
            .snapshot
            .assembly_steps
            .iter()
            .filter(|step| step.status == AssemblyStepStatus::Succeeded)
            .flat_map(|step| step.provides.iter().cloned())
            .collect();

        for capability in &mut self.snapshot.capabilities {
            if completed.contains(&capability.name) {
                capability.status = phenome_domain::CapabilityStatus::Ready;
            } else if capability.status == phenome_domain::CapabilityStatus::Ready {
                capability.status = phenome_domain::CapabilityStatus::Degraded;
            }
        }
    }
}

fn assembly_step_from_def(def: &AssemblyStepDef) -> AssemblyStep {
    AssemblyStep {
        id: def.id.clone(),
        kind: def.kind.clone(),
        depends_on: def.depends_on.clone(),
        provides: def.provides.clone(),
        status: AssemblyStepStatus::Pending,
        domain: def.domain.clone(),
        pod: def.pod.clone(),
    }
}

impl Default for Runtime {
    fn default() -> Self {
        Self::new(ActionRegistry::default())
    }
}
