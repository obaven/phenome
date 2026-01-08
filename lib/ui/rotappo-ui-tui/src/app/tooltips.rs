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
                        "Assembly: {}/{} complete",
                        snapshot.assembly.completed, snapshot.assembly.total
                    ),
                    format!("In progress: {}", snapshot.assembly.in_progress),
                    format!("Blocked: {}", snapshot.assembly.blocked),
                    format!("Pending: {}", snapshot.assembly.pending),
                    format!("Health: {}", snapshot.health.as_str()),
                    format!(
                        "Last action: {}",
                        snapshot
                            .last_action
                            .map(|action| action.to_string())
                            .unwrap_or_else(|| "none".to_string())
                    ),
                ],
            });
        }
        if let Some(index) = self.ui.hover_action_index {
            let step = self.runtime.snapshot().assembly_steps.get(index)?;
            return Some(Tooltip {
                title: format!("Step {}", step.id),
                lines: vec![
                    format!("Domain: {}", step.domain),
                    format!("Status: {}", step.status.as_str()),
                    format!("Kind: {}", step.kind),
                    format!("Depends: {}", step.depends_on.len()),
                    format!("Provides: {}", step.provides.len()),
                    format!("Pod: {}", step.pod.as_deref().unwrap_or("-")),
                ],
            });
        }
        if let Some(index) = self.ui.hover_capability_index {
            let cap = self.runtime.snapshot().capabilities.get(index)?;
            return Some(Tooltip {
                title: format!("Capability {}", cap.name),
                lines: vec![format!("Status: {}", cap.status.as_str())],
            });
        }
        if let Some(index) = self.ui.hover_action_index {
            let action = self.runtime.registry().actions().get(index)?;
            return Some(Tooltip {
                title: format!("Action {}", action.label),
                lines: vec![
                    format!("ID: {}", action.id),
                    format!("Safety: {}", action.safety.as_str()),
                    format!("Confirm: {}", action.requires_confirmation),
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
