use super::BootstrapApp;

const LOG_MAX_EVENTS: usize = 500;

impl BootstrapApp {
    pub(crate) fn refresh_logs(&mut self) {
        let new_events = self.ports.logs.drain_events();
        if new_events.is_empty() {
            return;
        }

        let at_bottom = self.ui.log_scroll == 0;
        if !at_bottom {
            self.ui.log_scroll = self.ui.log_scroll.saturating_add(new_events.len());
        }

        for event in new_events {
            self.ui.log_events.push_back(event);
        }
        while self.ui.log_events.len() > LOG_MAX_EVENTS {
            self.ui.log_events.pop_front();
        }

        if at_bottom {
            self.ui.log_scroll = 0;
        } else {
            self.ui.log_scroll = self.ui.log_scroll.min(self.max_log_offset());
        }
    }

    pub(crate) fn scroll_logs(&mut self, delta: i32) {
        let max_offset = self.max_log_offset();
        if delta.is_positive() {
            self.ui.log_scroll = self.ui.log_scroll.saturating_add(delta as usize);
        } else if delta.is_negative() {
            self.ui.log_scroll = self
                .ui
                .log_scroll
                .saturating_sub(delta.unsigned_abs() as usize);
        }
        self.ui.log_scroll = self.ui.log_scroll.min(max_offset);
    }

    pub(crate) fn max_log_offset(&self) -> usize {
        let view_height = self.ui.log_view_height.max(1);
        self.ui.log_events.len().saturating_sub(view_height)
    }
}
