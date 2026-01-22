use ratatui::layout::Margin;

use crate::app::NavView;

use crate::app::App;

impl App {
    pub fn scroll_actions(&mut self, delta: i16) {
        let actions = self.runtime.registry().actions();
        let total = actions.len() as i16;
        let margin = if matches!(self.active_view(), NavView::TerminalCommands) {
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
        let item_height = if matches!(self.active_view(), NavView::TerminalCommands) {
            1
        } else {
            2
        };
        let visible = (inner.height as usize / item_height).max(1) as i16;
        let max_offset = total.saturating_sub(visible).max(0) as u16;
        let next = if delta.is_positive() {
            self.ui.actions_scroll.saturating_add(delta as u16)
        } else {
            self.ui.actions_scroll.saturating_sub(delta.unsigned_abs())
        };
        self.ui.actions_scroll = next.min(max_offset);
    }

    pub fn sync_action_scroll(&mut self, selected: usize) {
        let margin = if matches!(self.active_view(), NavView::TerminalCommands) {
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
        let item_height = if matches!(self.active_view(), NavView::TerminalCommands) {
            1
        } else {
            2
        };
        let visible = (inner.height as usize / item_height).max(1);
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
