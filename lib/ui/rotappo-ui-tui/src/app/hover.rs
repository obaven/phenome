use ratatui::layout::Margin;

use crate::state::HoverPanel;
use crate::util::{assembly_lines, collect_problems};

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
        self.ui.hover_node_id = None;

        if self.ui.snapshot_area.contains(pos) && !self.ui.collapsed_snapshot {
            self.ui.hover_snapshot = true;
        }

        if self.ui.assembly_area.contains(pos) && !self.ui.collapsed_assembly_steps {
            let view = self.active_view();
            if matches!(
                view,
                crate::app::NavView::TopologyDagGraph | crate::app::NavView::TopologyDualGraph
            ) {
                // Graph hover logic
                // Calculate graph coordinates
                if let Some(bounds) = self.graph.view_bounds(self.ui.assembly_area) {
                    let area = self.ui.assembly_area;
                    let width = area.width.max(1) as f64;
                    let height = area.height.max(1) as f64;
                    // Use center of cell (+0.5) for better accuracy
                    let x_ratio = (column.saturating_sub(area.x) as f64 + 0.5) / width;
                    let y_ratio = (row.saturating_sub(area.y) as f64 + 0.5) / height;
                    let x = bounds.x_min + x_ratio * (bounds.x_max - bounds.x_min);
                    let y = bounds.y_max - y_ratio * (bounds.y_max - bounds.y_min);

                    self.ui.hover_node_id = self.graph.node_id_at(x, y);
                    if self.ui.hover_node_id.is_some() {
                        self.ui.hover_panel = HoverPanel::Graph;
                    }
                }
            } else {
                self.ui.hover_panel = HoverPanel::Assembly;
                self.ui.hover_action_index = self.hover_index_in_assembly(row);
            }
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
        let margin = if matches!(self.active_view(), crate::app::NavView::TerminalCommands) {
            Margin {
                horizontal: 0,
                vertical: 0,
            }
        } else {
            Margin {
                horizontal: 1,
                vertical: 1,
            }
        };
        let inner = self.ui.actions_area.inner(margin);
        if inner.height == 0 || row < inner.y || row >= inner.y + inner.height {
            return None;
        }
        let offset = row.saturating_sub(inner.y) as usize;
        let item_height = if matches!(self.active_view(), crate::app::NavView::TerminalCommands) {
            1usize
        } else {
            2usize
        };
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
