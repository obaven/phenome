use ratatui::layout::Rect;

use super::{App, PanelId};

impl App {
    pub fn handle_header_click(&mut self, column: u16, row: u16) -> bool {
        if self.ui.settings_gear_area.contains((column, row).into()) {
            self.toggle_settings_panel();
            return true;
        }
        self.toggle_if_header(
            PanelId::AssemblyProgress,
            self.ui.assembly_progress_area,
            column,
            row,
        ) || self.toggle_if_header(PanelId::Snapshot, self.ui.snapshot_area, column, row)
            || self.toggle_if_header(
                PanelId::Capabilities,
                self.ui.capabilities_area,
                column,
                row,
            )
            || self.toggle_if_header(PanelId::AssemblySteps, self.ui.assembly_area, column, row)
            || self.toggle_if_header(PanelId::Actions, self.ui.actions_area, column, row)
            || self.toggle_if_header(PanelId::Settings, self.ui.settings_area, column, row)
            || self.toggle_if_header(PanelId::LogControls, self.ui.log_controls_area, column, row)
            || self.toggle_if_header(PanelId::Problems, self.ui.problems_area, column, row)
            || self.toggle_if_header(PanelId::Logs, self.ui.logs_area, column, row)
            || self.toggle_if_header(
                PanelId::Notifications,
                self.ui.notifications_area,
                column,
                row,
            )
            || self.toggle_if_header(PanelId::Help, self.ui.help_area, column, row)
    }

    fn toggle_if_header(&mut self, panel: PanelId, area: Rect, column: u16, row: u16) -> bool {
        if area.height < 1 {
            return false;
        }
        if area.contains((column, row).into()) && row == area.y {
            self.toggle_panel(panel);
            return true;
        }
        false
    }

    fn toggle_panel(&mut self, panel: PanelId) {
        if self.panel_collapsed(panel) {
            self.open_panel(panel);
        } else {
            self.close_panel(panel);
        }
    }

    pub fn toggle_settings_panel(&mut self) {
        self.toggle_panel(PanelId::Settings);
    }

    pub fn toggle_notifications_panel(&mut self) {
        self.toggle_panel(PanelId::Notifications);
    }

    pub(crate) fn is_collapsed(&self, panel: PanelId) -> bool {
        match panel {
            PanelId::AssemblyProgress => self.ui.collapsed_assembly_progress,
            PanelId::Snapshot => self.ui.collapsed_snapshot,
            PanelId::Capabilities => self.ui.collapsed_capabilities,
            PanelId::AssemblySteps => self.ui.collapsed_assembly_steps,
            PanelId::Actions => self.ui.collapsed_actions,
            PanelId::Settings => self.ui.collapsed_settings,
            PanelId::LogControls => self.ui.collapsed_log_controls,
            PanelId::Problems => self.ui.collapsed_problems,
            PanelId::Logs => self.ui.collapsed_logs,
            PanelId::Help => self.ui.collapsed_help,
            PanelId::Notifications => self.ui.collapsed_notifications,
            PanelId::Navbar => false,
        }
    }

    fn set_ui_collapsed(&mut self, panel: PanelId, value: bool) {
        match panel {
            PanelId::AssemblyProgress => self.ui.collapsed_assembly_progress = value,
            PanelId::Snapshot => self.ui.collapsed_snapshot = value,
            PanelId::Capabilities => self.ui.collapsed_capabilities = value,
            PanelId::AssemblySteps => self.ui.collapsed_assembly_steps = value,
            PanelId::Actions => self.ui.collapsed_actions = value,
            PanelId::Settings => self.ui.collapsed_settings = value,
            PanelId::LogControls => self.ui.collapsed_log_controls = value,
            PanelId::Problems => self.ui.collapsed_problems = value,
            PanelId::Logs => self.ui.collapsed_logs = value,
            PanelId::Help => self.ui.collapsed_help = value,
            PanelId::Notifications => self.ui.collapsed_notifications = value,
            PanelId::Navbar => {}
        }
    }

    fn open_panel(&mut self, panel: PanelId) {
        if let Some(slot) = panel.slot_id() {
            self.layout_policy.request_open(slot, self.ui.screen_area);
        }
        self.sync_ui_collapsed_from_policy();
        if !self.panel_collapsed(panel) {
            self.mark_opened(panel);
        }
    }

    fn close_panel(&mut self, panel: PanelId) {
        if let Some(slot) = panel.slot_id() {
            self.layout_policy.request_close(slot);
        }
        self.sync_ui_collapsed_from_policy();
    }

    fn mark_opened(&mut self, panel: PanelId) {
        self.ui.open_counter = self.ui.open_counter.wrapping_add(1);
        match panel {
            PanelId::Help => self.ui.help_opened_at = Some(self.ui.open_counter),
            PanelId::Logs => self.ui.logs_opened_at = Some(self.ui.open_counter),
            _ => {}
        }
    }

    pub fn sync_ui_collapsed_from_policy(&mut self) {
        let panels = [
            PanelId::AssemblyProgress,
            PanelId::Snapshot,
            PanelId::Capabilities,
            PanelId::AssemblySteps,
            PanelId::Actions,
            PanelId::Settings,
            PanelId::LogControls,
            PanelId::Problems,
            PanelId::Logs,
            PanelId::Help,
            PanelId::Notifications,
        ];
        for panel in panels {
            if let Some(slot) = panel.slot_id() {
                if let Some(collapsed) = self.layout_policy.collapsed_for(slot) {
                    self.set_ui_collapsed(panel, collapsed);
                }
            }
        }
    }

    pub fn middle_aux_panel(&self) -> Option<PanelId> {
        if !self.panel_collapsed(PanelId::AssemblySteps) {
            return None;
        }
        let help_open = !self.panel_collapsed(PanelId::Help);
        let logs_open = !self.panel_collapsed(PanelId::Logs);
        match (help_open, logs_open) {
            (true, false) => Some(PanelId::Help),
            (false, true) => Some(PanelId::Logs),
            (true, true) => {
                let help_at = self.ui.help_opened_at.unwrap_or(u64::MAX);
                let logs_at = self.ui.logs_opened_at.unwrap_or(u64::MAX);
                if help_at <= logs_at {
                    Some(PanelId::Help)
                } else {
                    Some(PanelId::Logs)
                }
            }
            _ => None,
        }
    }
}
