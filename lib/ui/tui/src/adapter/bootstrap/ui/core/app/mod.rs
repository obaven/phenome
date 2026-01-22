use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::prelude::Frame;

use crate::bootstrap::panels::{
    dependency_tree, header, logs as logs_panel, menu as menu_panel, status, summary,
};
use crate::bootstrap::state::BootstrapUiState;
use phenome_ports::PortSet;

mod input;
mod log_logic;
mod menu;
mod navigation;

pub struct BootstrapApp {
    pub ports: PortSet,
    pub ui: BootstrapUiState,
    pub should_quit: bool,
}

impl BootstrapApp {
    pub fn new(ports: PortSet) -> Self {
        Self {
            ports,
            ui: BootstrapUiState::default(),
            should_quit: false,
        }
    }

    pub fn on_tick(&mut self) {
        self.refresh_logs();
        let status = self.ports.bootstrap.bootstrap_status();
        if status.total_duration.is_some() && !self.ui.completed_seen {
            self.ui.show_summary = true;
            self.ui.completed_seen = true;
        }
    }

    pub fn render(&mut self, frame: &mut Frame) {
        if self.ui.show_summary {
            summary::render(frame, frame.area(), &self.ports);
        } else {
            let size = frame.area();
            let layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(5),
                    Constraint::Percentage(55),
                    Constraint::Percentage(45),
                ])
                .split(size);

            header::render(frame, layout[0], &self.ports);
            dependency_tree::render(frame, layout[1], &self.ports, &self.ui);
            status::render(frame, layout[2], &self.ports, &mut self.ui);

            if self.ui.menu_state.active {
                let actions = self.menu_actions();
                menu_panel::render(frame, size, &self.ports, &self.ui, &actions);
            }
        }

        if self.ui.show_logs {
            logs_panel::render(frame, frame.area(), &mut self.ui);
        }
    }
}
