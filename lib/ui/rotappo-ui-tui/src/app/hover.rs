use ratatui::layout::Margin;

use crate::state::HoverPanel;
use crate::util::{collect_problems, assembly_lines};

use super::App;

impl App {
    pub fn update_hover(&mut self, column: u16, row: u16) {
        let pos = (column, row).into();
        self.ui.mouse_pos = Some((column, row));
        if self.ui.log_menu_pinned && self.ui.log_menu_area.width > 0 {
            let in_menu = self.ui.log_menu_area.contains(pos);
            let in_trigger = self.log_menu_trigger_contains(pos);
            if !in_menu && !in_trigger {
                self.close_log_menu();
            }
        }
        self.ui.hover_panel = HoverPanel::None;
        self.ui.hover_action_index = None;
        self.ui.hover_capability_index = None;
        self.ui.hover_action_index = None;
        self.ui.hover_problem_index = None;
        self.ui.log_menu_hover_index = None;
        self.ui.hover_snapshot = false;

        if self.ui.snapshot_area.contains(pos) && !self.ui.collapsed_snapshot {
            self.ui.hover_snapshot = true;
        }

        if self.ui.assembly_area.contains(pos) && !self.ui.collapsed_assembly_steps {
            self.ui.hover_panel = HoverPanel::Assembly;
            self.ui.hover_action_index = self.hover_index_in_assembly(row);
        } else if self.ui.capabilities_area.contains(pos) && !self.ui.collapsed_capabilities {
            self.ui.hover_panel = HoverPanel::Capabilities;
            self.ui.hover_capability_index = self.hover_index_in_capabilities(row);
        } else if self.ui.actions_area.contains(pos) && !self.ui.collapsed_actions {
            self.ui.hover_panel = HoverPanel::Actions;
            self.ui.hover_action_index = self.hover_index_in_actions(row);
        } else if self.ui.log_controls_area.contains(pos) && !self.ui.collapsed_log_controls {
            self.ui.hover_panel = HoverPanel::Logs;
        } else if self.ui.settings_area.contains(pos) && !self.ui.collapsed_settings {
            self.ui.hover_panel = HoverPanel::Settings;
        } else if self.ui.logs_area.contains(pos) && !self.ui.collapsed_logs {
            self.ui.hover_panel = HoverPanel::Logs;
        } else if self.ui.problems_area.contains(pos) && !self.ui.collapsed_problems {
            self.ui.hover_panel = HoverPanel::Problems;
            self.ui.hover_problem_index = self.hover_index_in_problems(row);
        } else if self.ui.help_area.contains(pos) && !self.ui.collapsed_help {
            self.ui.hover_panel = HoverPanel::Help;
        } else if self.ui.log_menu_area.contains(pos) {
            self.ui.hover_panel = HoverPanel::Logs;
            self.ui.log_menu_hover_index = self.hover_index_in_log_menu(row);
        }
    }

    pub fn hover_index_in_assembly(&self, row: u16) -> Option<usize> {
        let inner = self.ui.assembly_area.inner(Margin {
            horizontal: 1,
            vertical: 1,
        });
        if inner.height == 0 || row < inner.y || row >= inner.y + inner.height {
            return None;
        }
        let offset = row.saturating_sub(inner.y) as usize;
        let lines = assembly_lines(self.runtime.snapshot());
        let line_index = offset + self.ui.assembly_scroll as usize;
        lines.get(line_index).and_then(|line| line.step_index)
    }

    pub fn hover_index_in_capabilities(&self, row: u16) -> Option<usize> {
        let inner = self.ui.capabilities_area.inner(Margin {
            horizontal: 1,
            vertical: 1,
        });
        if inner.height == 0 || row < inner.y || row >= inner.y + inner.height {
            return None;
        }
        let offset = row.saturating_sub(inner.y) as usize;
        let index = offset + self.ui.capabilities_scroll as usize;
        if index < self.runtime.snapshot().capabilities.len() {
            Some(index)
        } else {
            None
        }
    }

    pub fn hover_index_in_actions(&self, row: u16) -> Option<usize> {
        let inner = self.ui.actions_area.inner(Margin {
            horizontal: 1,
            vertical: 1,
        });
        if inner.height == 0 || row < inner.y || row >= inner.y + inner.height {
            return None;
        }
        let offset = row.saturating_sub(inner.y) as usize;
        let item_height = 2usize;
        let index = offset / item_height + self.ui.actions_scroll as usize;
        if index < self.runtime.registry().actions().len() {
            Some(index)
        } else {
            None
        }
    }

    pub fn hover_index_in_problems(&self, row: u16) -> Option<usize> {
        let inner = self.ui.problems_area.inner(Margin {
            horizontal: 1,
            vertical: 1,
        });
        if inner.height == 0 || row < inner.y || row >= inner.y + inner.height {
            return None;
        }
        let offset = row.saturating_sub(inner.y) as usize;
        let problems = collect_problems(self);
        if problems.is_empty() {
            return None;
        }
        let index = offset + self.ui.problems_scroll as usize;
        if index < problems.len() {
            Some(index)
        } else {
            None
        }
    }

    fn hover_index_in_log_menu(&self, row: u16) -> Option<usize> {
        let inner = self.ui.log_menu_area.inner(Margin {
            horizontal: 1,
            vertical: 1,
        });
        if inner.height == 0 || row < inner.y || row >= inner.y + inner.height {
            return None;
        }
        let offset = row.saturating_sub(inner.y) as usize;
        if offset < self.ui.log_menu_len {
            Some(offset)
        } else {
            None
        }
    }
}
