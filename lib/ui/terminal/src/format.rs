//! CLI formatting helpers for runtime data.

use anyhow::{Result, anyhow};
use serde::Serialize;

use phenome_domain::{ActionDefinition, Event, Snapshot};
use phenome_ui_presentation::formatting;

use super::OutputMode;

/// Format a collection of actions for CLI output.
///
/// # Examples
/// ```rust
/// use phenome_ui_terminal::{format_actions, OutputMode};
/// use phenome_domain::{ActionDefinition, ActionId, ActionSafety};
///
/// let actions = [ActionDefinition::new(
///     ActionId::Validate,
///     "Action Validate",
///     "Validate desired state.",
///     false,
///     ActionSafety::Safe,
/// )];
/// let output = format_actions(OutputMode::Plain, &actions).unwrap();
/// assert_eq!(output, "validate - Action Validate (safe)");
/// ```
pub fn format_actions(mode: OutputMode, actions: &[ActionDefinition]) -> Result<String> {
    match mode {
        OutputMode::Plain => Ok(actions
            .iter()
            .map(|action| {
                format!(
                    "{} - {} ({})",
                    action.id,
                    action.label,
                    action.safety.as_str()
                )
            })
            .collect::<Vec<_>>()
            .join("\n")),
        OutputMode::Json => Ok(serde_json::to_string_pretty(actions)?),
        OutputMode::Ndjson => to_ndjson(actions),
    }
}

/// Format a runtime snapshot for CLI output.
///
/// # Examples
/// ```rust
/// use phenome_ui_terminal::{format_snapshot, OutputMode};
/// use phenome_domain::Snapshot;
///
/// let snapshot = Snapshot::new_default();
/// let output = format_snapshot(OutputMode::Plain, &snapshot).unwrap();
/// assert_eq!(output, "Assembly 3/12 complete | Health: degraded");
/// ```
pub fn format_snapshot(mode: OutputMode, snapshot: &Snapshot) -> Result<String> {
    match mode {
        OutputMode::Plain => Ok(format!(
            "Assembly {}/{} complete | Health: {}",
            snapshot.assembly.completed,
            snapshot.assembly.total,
            snapshot.health.as_str()
        )),
        OutputMode::Json => Ok(serde_json::to_string_pretty(snapshot)?),
        OutputMode::Ndjson => to_ndjson(snapshot),
    }
}

/// Format event messages for CLI output.
///
/// # Examples
/// ```rust
/// use phenome_ui_terminal::{format_events, OutputMode};
/// use phenome_domain::{Event, EventLevel};
///
/// let events = [Event::new(EventLevel::Info, "ready")];
/// let output = format_events(OutputMode::Plain, &events).unwrap();
/// assert!(output.contains("info"));
/// assert!(output.contains("ready"));
/// ```
pub fn format_events(mode: OutputMode, events: &[Event]) -> Result<String> {
    match mode {
        OutputMode::Plain => Ok(events
            .iter()
            .map(|event| {
                format!(
                    "[{}] {} {}",
                    event.level.as_str(),
                    event.timestamp_ms,
                    event.message
                )
            })
            .collect::<Vec<_>>()
            .join("\n")),
        OutputMode::Json => Ok(serde_json::to_string_pretty(events)?),
        OutputMode::Ndjson => to_ndjson(events),
    }
}

/// Format the current assembly view for CLI output.
///
/// # Examples
/// ```rust
/// use phenome_ui_terminal::{format_assembly, OutputMode};
/// use phenome_domain::{HealthStatus, AssemblyStep, AssemblyStepStatus, AssemblySummary, Snapshot};
///
/// let snapshot = Snapshot {
///     assembly: AssemblySummary {
///         total: 1,
///         completed: 0,
///         in_progress: 1,
///         blocked: 0,
///         pending: 0,
///     },
///     assembly_steps: vec![AssemblyStep {
///         id: "boot".to_string(),
///         kind: "apply".to_string(),
///         depends_on: vec![],
///         provides: vec![],
///         status: AssemblyStepStatus::Running,
///         domain: "core".to_string(),
///         pod: None,
///     }],
///     capabilities: vec![],
///     health: HealthStatus::Healthy,
///     last_updated_ms: 0,
///     last_action: None,
///     last_action_status: None,
/// };
/// let output = format_assembly(OutputMode::Plain, &snapshot).unwrap();
/// assert!(output.contains("core domain"));
/// assert!(output.contains("boot apply"));
/// ```
pub fn format_assembly(mode: OutputMode, snapshot: &Snapshot) -> Result<String> {
    let groups = formatting::assembly_groups(snapshot);
    match mode {
        OutputMode::Plain => {
            let mut lines = Vec::new();
            for group in groups {
                lines.push(format!("{} domain", group.domain));
                for step_info in group.steps {
                    let step = step_info.step;
                    let pod_text = step
                        .pod
                        .as_deref()
                        .map(|pod| format!(" pod: {pod}"))
                        .unwrap_or_else(|| " pod: -".to_string());
                    lines.push(format!(
                        "[{:<9}] {} {}{}",
                        step.status.as_str(),
                        step.id,
                        step.kind,
                        pod_text
                    ));
                }
            }
            Ok(lines.join("\n"))
        }
        OutputMode::Json => Ok(serde_json::to_string_pretty(&groups)?),
        OutputMode::Ndjson => to_ndjson(&groups),
    }
}

/// Format problem strings for CLI output.
///
/// # Examples
/// ```rust
/// use phenome_ui_terminal::{format_problems, OutputMode};
///
/// let problems = vec!["one".to_string(), "two".to_string()];
/// let output = format_problems(OutputMode::Plain, &problems).unwrap();
/// assert_eq!(output, "one\ntwo");
/// ```
pub fn format_problems(mode: OutputMode, problems: &[String]) -> Result<String> {
    match mode {
        OutputMode::Plain => Ok(problems.join("\n")),
        OutputMode::Json => Ok(serde_json::to_string_pretty(problems)?),
        OutputMode::Ndjson => to_ndjson(problems),
    }
}

/// Serialize a value as single-line JSON suitable for NDJSON output.
fn to_ndjson<T: Serialize + ?Sized>(value: &T) -> Result<String> {
    let json = serde_json::to_string(value)?;
    if json.contains('\n') {
        return Err(anyhow!("NDJSON payload contains newlines"));
    }
    Ok(json)
}
