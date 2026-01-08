use ratatui::layout::Margin;

use crate::state::HoverPanel;
use crate::util::{collect_problems, assembly_lines};

use super::App;

impl App {
    pub fn scroll_active_panel(&mut self, delta: i16) {
        match self.ui.hover_panel {
            HoverPanel::Logs => self.scroll_logs(delta),
            HoverPanel::Assembly => self.scroll_assembly(delta),
            HoverPanel::Capabilities => self.scroll_capabilities(delta),
            HoverPanel::Actions => self.scroll_actions(delta),
            HoverPanel::Problems => self.scroll_problems(delta),
            _ => {
                if delta > 0 {
                    self.select_next_action();
                } else if delta < 0 {
                    self.select_previous_action();
                }
            }
        }
    }

    pub fn scroll_logs(&mut self, delta: i16) {
        let total = self.filtered_events().len() as i16;
        let max_offset = total.saturating_sub(1).max(0) as u16;
        let next = if delta.is_positive() {
            self.ui.log_scroll.saturating_add(delta as u16)
        } else {
            self.ui.log_scroll.saturating_sub(delta.unsigned_abs())
        };
        self.ui.log_scroll = next.min(max_offset);
    }

    pub fn scroll_assembly(&mut self, delta: i16) {
        let total = assembly_lines(self.runtime.snapshot()).len() as i16;
        let max_offset = total.saturating_sub(1).max(0) as u16;
        let next = if delta.is_positive() {
            self.ui.assembly_scroll.saturating_add(delta as u16)
        } else {
            self.ui.assembly_scroll
                .saturating_sub(delta.unsigned_abs())
        };
        self.ui.assembly_scroll = next.min(max_offset);
    }

    pub fn scroll_capabilities(&mut self, delta: i16) {
        let total = self.runtime.snapshot().capabilities.len() as i16;
        let max_offset = total.saturating_sub(1).max(0) as u16;
        let next = if delta.is_positive() {
            self.ui.capabilities_scroll.saturating_add(delta as u16)
        } else {
            self.ui.capabilities_scroll
                .saturating_sub(delta.unsigned_abs())
        };
        self.ui.capabilities_scroll = next.min(max_offset);
    }

    pub fn scroll_actions(&mut self, delta: i16) {
        let actions = self.runtime.registry().actions();
        let total = actions.len() as i16;
        let inner = self.ui.actions_area.inner(Margin {
            horizontal: 1,
            vertical: 1,
        });
        let visible = (inner.height as usize / 2).max(1) as i16;
        let max_offset = total.saturating_sub(visible).max(0) as u16;
        let next = if delta.is_positive() {
            self.ui.actions_scroll.saturating_add(delta as u16)
        } else {
            self.ui
                .actions_scroll
                .saturating_sub(delta.unsigned_abs())
        };
        self.ui.actions_scroll = next.min(max_offset);
    }

    pub fn scroll_problems(&mut self, delta: i16) {
        let total = collect_problems(self).len() as i16;
        let inner = self.ui.problems_area.inner(Margin {
            horizontal: 1,
            vertical: 1,
        });
        let visible = inner.height.max(1) as i16;
        let max_offset = total.saturating_sub(visible).max(0) as u16;
        let next = if delta.is_positive() {
            self.ui.problems_scroll.saturating_add(delta as u16)
        } else {
            self.ui
                .problems_scroll
                .saturating_sub(delta.unsigned_abs())
        };
        self.ui.problems_scroll = next.min(max_offset);
    }

    pub fn sync_action_scroll(&mut self, selected: usize) {
        let inner = self.ui.actions_area.inner(Margin {
            horizontal: 1,
            vertical: 1,
        });
        let visible = (inner.height as usize / 2).max(1);
        let max_offset = self
            .runtime
            .registry()
            .actions()
            .len()
            .saturating_sub(visible);
        if selected < self.ui.actions_scroll as usize {
            self.ui.actions_scroll = selected as u16;
        } else if selected >= self.ui.actions_scroll as usize + visible {
            let next = selected.saturating_sub(visible.saturating_sub(1));
            self.ui.actions_scroll = next.min(max_offset) as u16;
        }
    }
}
