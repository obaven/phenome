use crate::app::App;

impl App {
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
}
