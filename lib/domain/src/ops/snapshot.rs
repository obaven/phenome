use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

use super::actions::ActionId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AssemblyStepStatus {
    Pending,
    Running,
    Succeeded,
    Failed,
    Blocked,
}

impl AssemblyStepStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            AssemblyStepStatus::Pending => "pending",
            AssemblyStepStatus::Running => "running",
            AssemblyStepStatus::Succeeded => "completed",
            AssemblyStepStatus::Failed => "failed",
            AssemblyStepStatus::Blocked => "blocked",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssemblyStep {
    pub id: String,
    pub kind: String,
    pub depends_on: Vec<String>,
    pub provides: Vec<String>,
    pub status: AssemblyStepStatus,
    pub domain: String,
    pub pod: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssemblySummary {
    pub total: u32,
    pub completed: u32,
    pub in_progress: u32,
    pub blocked: u32,
    pub pending: u32,
}

impl AssemblySummary {
    pub fn percent_complete(&self) -> u16 {
        if self.total == 0 {
            return 0;
        }
        let ratio = self.completed.saturating_mul(100) / self.total;
        ratio.min(100) as u16
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unavailable,
}

impl HealthStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            HealthStatus::Healthy => "healthy",
            HealthStatus::Degraded => "degraded",
            HealthStatus::Unavailable => "unavailable",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CapabilityStatus {
    Ready,
    Degraded,
    Offline,
}

impl CapabilityStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            CapabilityStatus::Ready => "ready",
            CapabilityStatus::Degraded => "degraded",
            CapabilityStatus::Offline => "offline",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Capability {
    pub name: String,
    pub status: CapabilityStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActionStatus {
    Pending,
    Running,
    Succeeded,
    Failed,
}

impl ActionStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            ActionStatus::Pending => "pending",
            ActionStatus::Running => "running",
            ActionStatus::Succeeded => "succeeded",
            ActionStatus::Failed => "failed",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot {
    pub assembly: AssemblySummary,
    pub assembly_steps: Vec<AssemblyStep>,
    pub capabilities: Vec<Capability>,
    pub health: HealthStatus,
    pub last_updated_ms: u64,
    pub last_action: Option<ActionId>,
    pub last_action_status: Option<ActionStatus>,
}

impl Snapshot {
    pub fn new_default() -> Self {
        Self {
            assembly: AssemblySummary {
                total: 12,
                completed: 3,
                in_progress: 2,
                blocked: 1,
                pending: 6,
            },
            assembly_steps: vec![],
            capabilities: vec![
                Capability {
                    name: "Assembly Snapshot".to_string(),
                    status: CapabilityStatus::Ready,
                },
                Capability {
                    name: "Assembly Router".to_string(),
                    status: CapabilityStatus::Degraded,
                },
                Capability {
                    name: "Event Stream".to_string(),
                    status: CapabilityStatus::Ready,
                },
                Capability {
                    name: "Desktop Bridge".to_string(),
                    status: CapabilityStatus::Offline,
                },
            ],
            health: HealthStatus::Degraded,
            last_updated_ms: now_millis(),
            last_action: None,
            last_action_status: None,
        }
    }

    pub fn touch(&mut self) {
        self.last_updated_ms = now_millis();
    }

    pub fn mark_action(&mut self, action: ActionId, status: ActionStatus) {
        self.last_action = Some(action);
        self.last_action_status = Some(status);
        self.touch();
    }

    pub fn update_assembly_summary_from_steps(&mut self) {
        if self.assembly_steps.is_empty() {
            return;
        }
        let total = self.assembly_steps.len() as u32;
        let mut completed = 0;
        let mut in_progress = 0;
        let mut blocked = 0;
        let mut pending = 0;
        for step in &self.assembly_steps {
            match step.status {
                AssemblyStepStatus::Succeeded => completed += 1,
                AssemblyStepStatus::Running => in_progress += 1,
                AssemblyStepStatus::Blocked => blocked += 1,
                AssemblyStepStatus::Pending => pending += 1,
                _ => {}
            }
        }
        self.assembly.total = total;
        self.assembly.completed = completed;
        self.assembly.in_progress = in_progress;
        self.assembly.blocked = blocked;
        self.assembly.pending = pending;
    }
}

pub fn now_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}
