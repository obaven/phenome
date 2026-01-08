//! Domain models and invariants.

pub mod assembly;
pub mod actions;
pub mod events;
pub mod health;
pub mod snapshot;

pub use assembly::{
    Assembly, AssemblyStepDef,
};
pub use actions::{
    ActionDefinition, ActionId, ActionRegistry, ActionSafety,
};
pub use events::{Event, EventBus, EventLevel};
pub use health::{ComponentHealthStatus, HealthSnapshot};
pub use snapshot::{
    now_millis, ActionStatus, Capability, CapabilityStatus, HealthStatus, AssemblyStep,
    AssemblyStepStatus, AssemblySummary, Snapshot,
};
