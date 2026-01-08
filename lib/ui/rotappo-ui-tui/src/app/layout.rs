//! Layout-driven helpers for panel sizing and visibility.

use ratatui::layout::{Margin, Position, Rect};

use super::{
    App, PanelId, COLLAPSED_HEIGHT, FILTER_LABEL, LOG_CONTROLS_BASE_HEIGHT,
    LOG_MENU_FILTER_LEN, LOG_MENU_STREAM_LEN, STREAM_LABEL,
};

impl App {
    /// Close the log menu and reset its hover state.
    pub fn close_log_menu(&mut self) {
        self.ui.log_menu_pinned = false;
        self.ui.log_menu_mode = None;
        self.ui.log_menu_len = 0;
        self.ui.log_menu_area = Rect::default();
        self.ui.log_menu_hover_index = None;
    }

    /// Sync layout policy overrides with the UI's collapsed state.
    pub fn sync_layout_policy(&mut self) {
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
        ];
        for panel in panels {
            if let Some(slot) = panel.slot_id() {
                let slot_id = slot.into();
                self.layout_policy.clear_override(&slot_id);
                self.layout_policy
                    .set_collapsed(slot, self.is_collapsed(panel));
            }
        }
    }

    /// Compute log controls height based on menu visibility.
    pub fn log_controls_height(&self) -> u16 {
        if self.panel_collapsed(PanelId::LogControls) {
            return COLLAPSED_HEIGHT;
        }
        let menu_items = if self.ui.log_menu_pinned {
            match self.ui.log_menu_mode {
                Some(crate::state::LogMenuMode::Filter) => LOG_MENU_FILTER_LEN,
                Some(crate::state::LogMenuMode::Stream) => LOG_MENU_STREAM_LEN,
                None => 0,
            }
        } else {
            0
        };
        let menu_height = if menu_items > 0 { menu_items + 2 } else { 0 };
        LOG_CONTROLS_BASE_HEIGHT.saturating_add(menu_height)
    }

    /// Resolve whether a panel is visible after layout policy overrides.
    pub fn panel_visible(&self, panel: PanelId) -> bool {
        if let Some(slot) = panel.slot_id() {
            if let Some(visible) = self.layout_policy.visibility_for(slot) {
                return visible;
            }
        }
        true
    }

    /// Resolve whether a panel is collapsed after layout policy overrides.
    pub fn panel_collapsed(&self, panel: PanelId) -> bool {
        if let Some(slot) = panel.slot_id() {
            if let Some(collapsed) = self.layout_policy.collapsed_for(slot) {
                return collapsed;
            }
        }
        self.is_collapsed(panel)
    }

    pub(crate) fn log_menu_trigger_contains(&self, pos: Position) -> bool {
        if self.ui.log_filter_tag_area.contains(pos)
            || self.ui.log_stream_tag_area.contains(pos)
        {
            return true;
        }
        if self.ui.log_controls_area.height == 0 {
            return false;
        }
        let inner = self.ui.log_controls_area.inner(Margin {
            horizontal: 1,
            vertical: 1,
        });
        if inner.height == 0 || pos.y != inner.y {
            return false;
        }
        let filter_start =
            self.ui.log_filter_tag_area.x.saturating_sub(FILTER_LABEL.len() as u16);
        let filter_end = self
            .ui
            .log_filter_tag_area
            .x
            .saturating_add(self.ui.log_filter_tag_area.width);
        let stream_start =
            self.ui.log_stream_tag_area.x.saturating_sub(STREAM_LABEL.len() as u16);
        let stream_end = self
            .ui
            .log_stream_tag_area
            .x
            .saturating_add(self.ui.log_stream_tag_area.width);
        let x = pos.x;
        (x >= filter_start && x < filter_end) || (x >= stream_start && x < stream_end)
    }
}
