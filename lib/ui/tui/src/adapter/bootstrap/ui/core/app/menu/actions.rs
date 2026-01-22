use phenome_ports::ComponentStatus;

use crate::bootstrap::state::MenuAction;

use super::super::BootstrapApp;

impl BootstrapApp {
    pub(crate) fn selected_component_id(&self) -> Option<String> {
        let steps = self.ports.bootstrap.dependency_graph().steps.as_slice();
        steps
            .get(self.ui.status_selected)
            .map(|step| step.id.clone())
    }

    pub(crate) fn menu_actions(&self) -> Vec<MenuAction> {
        let Some(component) = self.selected_component_id() else {
            return vec![MenuAction::Cancel];
        };
        let state = self.ports.bootstrap.component_states();
        let status = state.get(&component).map(|s| s.status);

        let mut actions = Vec::new();
        match status {
            Some(ComponentStatus::Running) => {
                actions.push(MenuAction::Skip);
                actions.push(MenuAction::AdjustTimeout);
                actions.push(MenuAction::ViewLogs);
                actions.push(MenuAction::ToggleExpand);
            }
            Some(ComponentStatus::Failed) => {
                actions.push(MenuAction::Retry);
                actions.push(MenuAction::ViewLogs);
                actions.push(MenuAction::ToggleExpand);
            }
            Some(ComponentStatus::Deferred) => actions.push(MenuAction::Retry),
            Some(ComponentStatus::Complete) => {
                actions.push(MenuAction::ViewLogs);
                actions.push(MenuAction::ToggleExpand);
            }
            _ => {}
        }

        if self.ui.paused {
            actions.push(MenuAction::Resume);
        } else {
            actions.push(MenuAction::Pause);
        }
        actions.push(MenuAction::Cancel);

        actions
    }
}
