use anyhow::{Result, anyhow};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap},
};
use rotappo_ports::{ComponentStatus, InteractiveCommand, PortSet};

#[derive(Debug, Clone)]
enum MenuAction {
    SkipComponent,
    RetryComponent,
    AdjustTimeout,
    ViewLogs,
    ExpandDetails,
    PauseBootstrap,
    ResumeBootstrap,
    CancelBootstrap,
}

impl ToString for MenuAction {
    fn to_string(&self) -> String {
        match self {
            Self::SkipComponent => "Skip Component".to_string(),
            Self::RetryComponent => "Retry Component".to_string(),
            Self::AdjustTimeout => "Adjust Timeout".to_string(),
            Self::ViewLogs => "View Logs".to_string(),
            Self::ExpandDetails => "Expand Details".to_string(),
            Self::PauseBootstrap => "Pause Bootstrap".to_string(),
            Self::ResumeBootstrap => "Resume Bootstrap".to_string(),
            Self::CancelBootstrap => "Cancel Bootstrap".to_string(),
        }
    }
}

pub struct InteractiveMenuPanel {
    active: bool,
    list_state: ListState,
    items: Vec<MenuAction>,
    current_component: Option<String>,
    show_confirmation: bool,
    confirmation_message: String,
    pending_command: Option<InteractiveCommand>,
}

impl Default for InteractiveMenuPanel {
    fn default() -> Self {
        Self {
            active: false,
            list_state: ListState::default(),
            items: Vec::new(),
            current_component: None,
            show_confirmation: false,
            confirmation_message: String::new(),
            pending_command: None,
        }
    }
}

impl InteractiveMenuPanel {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_active(&mut self, active: bool) {
        self.active = active;
        if active {
            // Reset state when opening
            self.list_state.select(Some(0));
            self.show_confirmation = false;
            self.pending_command = None;
        }
    }

    pub fn set_current_component(&mut self, component_id: Option<String>) {
        self.current_component = component_id;
    }

    fn build_menu_items(&self, ports: &PortSet) -> Vec<MenuAction> {
        let mut actions = Vec::new();
        let bootstrap_port = &ports.bootstrap;

        let states = bootstrap_port.component_states();
        let component_status = self
            .current_component
            .as_ref()
            .and_then(|id| states.get(id));

        if let Some(status) = component_status {
            match status.status {
                ComponentStatus::Running | ComponentStatus::Pending => {
                    actions.push(MenuAction::SkipComponent);
                    actions.push(MenuAction::AdjustTimeout);
                    actions.push(MenuAction::ViewLogs);
                    actions.push(MenuAction::ExpandDetails);
                }
                ComponentStatus::Failed => {
                    actions.push(MenuAction::RetryComponent);
                    actions.push(MenuAction::ViewLogs);
                    actions.push(MenuAction::ExpandDetails);
                }
                ComponentStatus::Deferred => {
                    actions.push(MenuAction::RetryComponent);
                }
                ComponentStatus::Complete => {
                    actions.push(MenuAction::ViewLogs);
                    actions.push(MenuAction::ExpandDetails);
                }
            }
        } else if self.current_component.is_some() {
            // Fallback if status not found but component selected
            actions.push(MenuAction::SkipComponent);
        }

        // Global actions
        // TODO: detecting paused state would require exposing it on the port
        actions.push(MenuAction::PauseBootstrap);
        actions.push(MenuAction::ResumeBootstrap);
        actions.push(MenuAction::CancelBootstrap);

        actions
    }

    fn render_menu(&mut self, f: &mut Frame, area: Rect, ports: &PortSet) {
        self.items = self.build_menu_items(ports);

        // Ensure selection is valid
        if self.list_state.selected().is_none() && !self.items.is_empty() {
            self.list_state.select(Some(0));
        }

        let list_items: Vec<ListItem> = self
            .items
            .iter()
            .map(|action| {
                ListItem::new(Line::from(vec![
                    Span::raw(" > "),
                    Span::styled(action.to_string(), Style::default().bold()),
                ]))
            })
            .collect();

        let title = if let Some(id) = &self.current_component {
            format!(" Menu: {} ", id)
        } else {
            " Menu ".to_string()
        };

        let menu = List::new(list_items)
            .block(Block::default().borders(Borders::ALL).title(title))
            .highlight_style(
                Style::default()
                    .bg(Color::Blue)
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            );

        f.render_stateful_widget(menu, area, &mut self.list_state);
    }

    fn render_confirmation(&self, f: &mut Frame, area: Rect) {
        let block = Block::default()
            .title(" Confirm Action ")
            .borders(Borders::ALL)
            .style(Style::default().bg(Color::Red).fg(Color::White));

        let text = Paragraph::new(self.confirmation_message.clone())
            .block(block)
            .wrap(Wrap { trim: true })
            .alignment(ratatui::layout::Alignment::Center);

        f.render_widget(text, area);
    }

    fn execute_selected_action(&mut self, ports: &PortSet) -> Result<()> {
        let selected_index = self.list_state.selected().unwrap_or(0);
        let action = self
            .items
            .get(selected_index)
            .ok_or_else(|| anyhow!("No action selected"))?;

        match action {
            MenuAction::SkipComponent => {
                let id = self
                    .current_component
                    .clone()
                    .ok_or_else(|| anyhow!("No component selected"))?;
                self.show_confirmation = true;
                // Ideally we'd show dependents here, but that requires querying the port/assembly which might not be exposed on the port easily.
                // For now, simpler message.
                self.confirmation_message = format!(
                    "Are you sure you want to SKIP component '{}'?\n\nThis will also defer any components that depend on it.\n\nPress 'y' to confirm, 'n' to cancel.",
                    id
                );
                self.pending_command = Some(InteractiveCommand::SkipComponent { id });
            }
            MenuAction::RetryComponent => {
                let id = self
                    .current_component
                    .clone()
                    .ok_or_else(|| anyhow!("No component selected"))?;
                ports
                    .bootstrap
                    .send_command(InteractiveCommand::RetryComponent { id })?;
                self.active = false; // Close menu after action
            }
            MenuAction::AdjustTimeout => {
                // TODO: Implement Input Dialog for timeout
                // For now just logging placeholder
                // self.active = false;
            }
            MenuAction::PauseBootstrap => {
                ports
                    .bootstrap
                    .send_command(InteractiveCommand::PauseBootstrap)?;
                self.active = false;
            }
            MenuAction::ResumeBootstrap => {
                ports
                    .bootstrap
                    .send_command(InteractiveCommand::ResumeBootstrap)?;
                self.active = false;
            }
            MenuAction::CancelBootstrap => {
                self.show_confirmation = true;
                self.confirmation_message = "Are you sure you want to CANCEL the entire bootstrap process?\n\nPress 'y' to confirm, 'n' to cancel.".to_string();
                self.pending_command = Some(InteractiveCommand::CancelBootstrap);
            }
            _ => {}
        }
        Ok(())
    }

    pub fn render(&mut self, f: &mut Frame, area: Rect, ports: &PortSet) {
        if !self.active {
            return;
        }

        let overlay_area = centered_rect(60, 40, area);

        // Clear area behind popup
        f.render_widget(Clear, overlay_area);

        if self.show_confirmation {
            self.render_confirmation(f, overlay_area);
        } else {
            self.render_menu(f, overlay_area, ports);
        }
    }

    pub fn handle_input(&mut self, key: KeyEvent, ports: &PortSet) -> Result<()> {
        if !self.active {
            return Ok(());
        }

        if self.show_confirmation {
            match key.code {
                KeyCode::Char('y') | KeyCode::Char('Y') => {
                    if let Some(cmd) = self.pending_command.take() {
                        ports.bootstrap.send_command(cmd)?;
                    }
                    self.show_confirmation = false;
                    self.active = false; // Close menu after confirmed action
                }
                KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                    self.show_confirmation = false;
                    self.pending_command = None;
                }
                _ => {}
            }
            return Ok(());
        }

        match key.code {
            KeyCode::Up => {
                let i = match self.list_state.selected() {
                    Some(i) => {
                        if i == 0 {
                            self.items.len() - 1
                        } else {
                            i - 1
                        }
                    }
                    None => 0,
                };
                self.list_state.select(Some(i));
            }
            KeyCode::Down => {
                let i = match self.list_state.selected() {
                    Some(i) => {
                        if i >= self.items.len() - 1 {
                            0
                        } else {
                            i + 1
                        }
                    }
                    None => 0,
                };
                self.list_state.select(Some(i));
            }
            KeyCode::Enter => {
                self.execute_selected_action(ports)?;
            }
            KeyCode::Esc => {
                self.active = false;
            }
            _ => {}
        }
        Ok(())
    }
}

/// Helper function to center a rect
/// Helper function to center a rect
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;
    use bootstrappo::application::timing::TimingHistory;
    use bootstrappo::domain::models::assembly::Assembly;
    use bootstrappo::domain::models::module::spec::ModuleSpec;
    use rotappo_ports::{
        AccessUrlInfo, BootstrapPort, BootstrapStatus, ComponentState, ComponentStatus, PortSet,
    };
    use std::collections::HashMap;
    use std::sync::Arc;

    struct MockBootstrapPort {
        states: HashMap<String, ComponentState>,
    }

    impl BootstrapPort for MockBootstrapPort {
        fn component_states(&self) -> HashMap<String, ComponentState> {
            self.states.clone()
        }
        fn dependency_graph(&self) -> &Assembly {
            static EMPTY: std::sync::OnceLock<Assembly> = std::sync::OnceLock::new();
            EMPTY.get_or_init(Assembly::default)
        }
        fn timing_history(&self) -> Option<TimingHistory> {
            None
        }
        fn bootstrap_status(&self) -> BootstrapStatus {
            BootstrapStatus::default()
        }
        fn access_urls(&self) -> Vec<AccessUrlInfo> {
            Vec::new()
        }
        fn send_command(&self, _cmd: InteractiveCommand) -> Result<()> {
            Ok(())
        }
        fn get_detailed_status(
            &self,
            _id: &str,
        ) -> Result<bootstrappo::application::readiness::DetailedStatus> {
            Ok(bootstrappo::application::readiness::DetailedStatus::empty())
        }
        fn registry_specs(&self) -> HashMap<String, ModuleSpec> {
            HashMap::new()
        }
    }

    fn create_test_ports(states: HashMap<String, ComponentState>) -> PortSet {
        let mut ports = PortSet::empty();
        ports.bootstrap = Arc::new(MockBootstrapPort { states });
        ports
    }

    #[test]
    fn test_menu_actions_running() {
        let mut panel = InteractiveMenuPanel::new();
        panel.set_current_component(Some("comp1".to_string()));

        let mut states = HashMap::new();
        let mut state = ComponentState::new("comp1".to_string());
        state.status = ComponentStatus::Running;
        states.insert("comp1".to_string(), state);

        let ports = create_test_ports(states);
        let items = panel.build_menu_items(&ports);

        assert!(items.iter().any(|a| matches!(a, MenuAction::SkipComponent)));
        assert!(items.iter().any(|a| matches!(a, MenuAction::AdjustTimeout)));
        assert!(items.iter().any(|a| matches!(a, MenuAction::ViewLogs)));
    }

    #[test]
    fn test_menu_actions_failed() {
        let mut panel = InteractiveMenuPanel::new();
        panel.set_current_component(Some("comp1".to_string()));

        let mut states = HashMap::new();
        let mut state = ComponentState::new("comp1".to_string());
        state.status = ComponentStatus::Failed;
        states.insert("comp1".to_string(), state);

        let ports = create_test_ports(states);
        let items = panel.build_menu_items(&ports);

        assert!(
            items
                .iter()
                .any(|a| matches!(a, MenuAction::RetryComponent))
        );
        assert!(!items.iter().any(|a| matches!(a, MenuAction::SkipComponent)));
    }

    #[test]
    fn test_global_actions_present() {
        let panel = InteractiveMenuPanel::new();
        let ports = create_test_ports(HashMap::new());
        let items = panel.build_menu_items(&ports);

        assert!(
            items
                .iter()
                .any(|a| matches!(a, MenuAction::PauseBootstrap))
        );
        assert!(
            items
                .iter()
                .any(|a| matches!(a, MenuAction::ResumeBootstrap))
        );
        assert!(
            items
                .iter()
                .any(|a| matches!(a, MenuAction::CancelBootstrap))
        );
    }
}
