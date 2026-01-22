use anyhow::Result;
use primer::application::events::InteractiveCommand;

use crate::bootstrap::state::MenuAction;
use crate::bootstrap::utils::find_dependents;

use super::super::BootstrapApp;

impl BootstrapApp {
    pub(crate) fn execute_menu_action(&mut self) -> Result<()> {
        let actions = self.menu_actions();
        let Some(action) = actions.get(self.ui.menu_state.selected).cloned() else {
            return Ok(());
        };

        match action {
            MenuAction::Skip => {
                let Some(component) = self.selected_component_id() else {
                    return Ok(());
                };
                let dependents =
                    find_dependents(self.ports.bootstrap.dependency_graph(), &component);
                self.ui.menu_state.confirm(
                    format!(
                        "Skip component: {}?\nThis will also defer: {}",
                        component,
                        if dependents.is_empty() {
                            "none".to_string()
                        } else {
                            dependents.join(", ")
                        }
                    ),
                    InteractiveCommand::SkipComponent { id: component },
                );
            }
            MenuAction::Retry => {
                if let Some(component) = self.selected_component_id() {
                    self.ports
                        .bootstrap
                        .send_command(InteractiveCommand::RetryComponent { id: component })?;
                }
                self.ui.menu_state.clear();
            }
            MenuAction::AdjustTimeout => {
                self.ui.menu_state.timeout_input = Some(String::new());
            }
            MenuAction::ViewLogs => {
                self.ui.show_logs = true;
                self.ui.log_scroll = 0;
                self.ui.menu_state.clear();
            }
            MenuAction::Pause => {
                self.ports
                    .bootstrap
                    .send_command(InteractiveCommand::PauseBootstrap)?;
                self.ui.paused = true;
                self.ui.menu_state.clear();
            }
            MenuAction::Resume => {
                self.ports
                    .bootstrap
                    .send_command(InteractiveCommand::ResumeBootstrap)?;
                self.ui.paused = false;
                self.ui.menu_state.clear();
            }
            MenuAction::Cancel => {
                self.ui.menu_state.confirm(
                    "Cancel bootstrap? This will stop execution.".to_string(),
                    InteractiveCommand::CancelBootstrap,
                );
            }
            MenuAction::ToggleExpand => {
                self.toggle_expand_selected();
                self.ui.menu_state.clear();
            }
        }

        Ok(())
    }
}
