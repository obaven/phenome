use crate::util::collect_problems;

use super::App;
use crate::state::Tooltip;

impl App {
    pub fn refresh_pulse_active(&self) -> bool {
        false
    }

    pub fn current_tooltip(&self) -> Option<Tooltip> {
        if self.ui.hover_snapshot {
            let snapshot = self.runtime.snapshot();
            return Some(Tooltip {
                title: "Snapshot".to_string(),
                lines: vec![
                    format!(
                        "Assembly: {completed}/{total} complete",
                        completed = snapshot.assembly.completed,
                        total = snapshot.assembly.total
                    ),
                    format!(
                        "In progress: {in_progress}",
                        in_progress = snapshot.assembly.in_progress
                    ),
                    format!("Blocked: {blocked}", blocked = snapshot.assembly.blocked),
                    format!("Pending: {pending}", pending = snapshot.assembly.pending),
                    format!("Health: {health}", health = snapshot.health.as_str()),
                    format!(
                        "Last action: {last_action}",
                        last_action = snapshot
                            .last_action
                            .map(|action| action.to_string())
                            .unwrap_or_else(|| "none".to_string())
                    ),
                ],
            });
        }
        if let Some(node_id) = &self.ui.hover_node_id {
            if let Some(step) = self
                .runtime
                .snapshot()
                .assembly_steps
                .iter()
                .find(|s| s.id == *node_id)
            {
                return Some(Tooltip {
                    title: format!("Step {id}", id = step.id),
                    lines: vec![
                        format!("Domain: {domain}", domain = step.domain.as_str()),
                        format!("Status: {status}", status = step.status.as_str()),
                        format!("Kind: {kind}", kind = step.kind.as_str()),
                        format!("Depends: {count}", count = step.depends_on.len()),
                        format!("Provides: {count}", count = step.provides.len()),
                        format!("Pod: {pod}", pod = step.pod.as_deref().unwrap_or("-")),
                    ],
                });
            } else {
                return Some(Tooltip {
                    title: format!("Node {}", node_id),
                    lines: vec![
                        "Details not found in assembly steps".to_string(),
                        format!("ID: {}", node_id),
                    ],
                });
            }
        }
        if let Some(index) = self.ui.hover_action_index {
            let step = self.runtime.snapshot().assembly_steps.get(index)?;
            return Some(Tooltip {
                title: format!("Step {id}", id = step.id.as_str()),
                lines: vec![
                    format!("Domain: {domain}", domain = step.domain.as_str()),
                    format!("Status: {status}", status = step.status.as_str()),
                    format!("Kind: {kind}", kind = step.kind.as_str()),
                    format!("Depends: {count}", count = step.depends_on.len()),
                    format!("Provides: {count}", count = step.provides.len()),
                    format!("Pod: {pod}", pod = step.pod.as_deref().unwrap_or("-")),
                ],
            });
        }
        if let Some(index) = self.ui.hover_capability_index {
            let cap = self.runtime.snapshot().capabilities.get(index)?;
            return Some(Tooltip {
                title: format!("Capability {name}", name = cap.name.as_str()),
                lines: vec![format!("Status: {status}", status = cap.status.as_str())],
            });
        }
        if let Some(index) = self.ui.hover_action_index {
            let action = self.runtime.registry().actions().get(index)?;
            return Some(Tooltip {
                title: format!("Action {label}", label = action.label),
                lines: vec![
                    format!("ID: {id}", id = action.id),
                    format!("Safety: {safety}", safety = action.safety.as_str()),
                    format!("Confirm: {confirm}", confirm = action.requires_confirmation),
                    action.description.to_string(),
                ],
            });
        }
        if let Some(index) = self.ui.hover_problem_index {
            let problems = collect_problems(self);
            if let Some(problem) = problems.get(index) {
                return Some(Tooltip {
                    title: "Problem".to_string(),
                    lines: vec![problem.clone()],
                });
            }
        }
        None
    }

    pub fn pin_tooltip(&mut self) {
        if let Some(tooltip) = self.current_tooltip() {
            self.ui.pinned_tooltip = Some(tooltip);
        }
    }

    pub fn unpin_tooltip(&mut self) {
        if self.ui.pinned_tooltip.is_some() {
            self.ui.pinned_tooltip = None;
        }
    }
}
