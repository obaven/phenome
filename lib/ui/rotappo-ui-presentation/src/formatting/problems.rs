//! Problem formatting: derive human-readable issues from health + assembly state.
//!
//! ## Responsibility
//! - Translate health/assembly signals into short, user-facing problem lines.
//! - Keep output stable for UI rendering and tests.
//!
//! ## Non-goals
//! - No sorting by severity; callers decide ordering if needed.
//! - No localization or structured error objects.
//!
//! ## Key invariants
//! - Output strings are concise and single-line.
//! - Assembly blocks are always included when present.
//!
//! ## Failure modes
//! - None; input data is treated as optional and defaults to safe messages.
//!
//! ## Performance notes
//! - Complexity: O(n) over components and assembly steps.
//!
//! ## Extension points
//! - Add new health status variants with matching labels.

use phenome_domain::{AssemblyStepStatus, ComponentHealthStatus, HealthSnapshot, Snapshot};

/// Build user-facing problem lines from a snapshot and optional health data.
///
/// ## Why
/// Provides a single place to normalize error status strings for the UI.
///
/// ## Inputs
/// - `snapshot`: assembly state with per-step status.
/// - `health`: optional live health snapshot (may be unavailable).
///
/// ## Output
/// - Vector of concise, display-ready problem strings.
pub fn problem_lines(snapshot: &Snapshot, health: Option<&HealthSnapshot>) -> Vec<String> {
    let mut problems = Vec::new();
    if let Some(health) = health {
        if let Some(error) = &health.last_error {
            problems.push(format!("kube: {error}"));
        }
        problems.extend(health_problem_lines(&health.health));
        if !health.cache_ready {
            problems.push("kube cache not ready".to_string());
        }
    } else {
        problems.push("live status disabled".to_string());
    }

    for step in &snapshot.assembly_steps {
        if step.status == AssemblyStepStatus::Blocked {
            problems.push(format!(
                "blocked: {} waiting on {:?}",
                step.id, step.depends_on
            ));
        }
    }

    problems
}

fn health_problem_lines(
    health: &std::collections::HashMap<String, ComponentHealthStatus>,
) -> Vec<String> {
    let mut problems = Vec::new();
    for (name, status) in health {
        match status {
            ComponentHealthStatus::Healthy => {}
            ComponentHealthStatus::Degraded(msg) => {
                problems.push(format!("{name} degraded: {msg}"))
            }
            ComponentHealthStatus::Unhealthy(msg) => {
                problems.push(format!("{name} unhealthy: {msg}"))
            }
        }
    }
    problems
}
