use crate::util::assembly_lines;

use crate::app::App;

impl App {
    pub fn scroll_assembly(&mut self, delta: i16) {
        let total = assembly_lines(self.runtime.snapshot()).len() as i16;
        let max_offset = total.saturating_sub(1).max(0) as u16;
        let next = if delta.is_positive() {
            self.ui.assembly_scroll.saturating_add(delta as u16)
        } else {
            self.ui.assembly_scroll.saturating_sub(delta.unsigned_abs())
        };
        self.ui.assembly_scroll = next.min(max_offset);
    }

    pub fn scroll_capabilities(&mut self, delta: i16) {
        let total = self.runtime.snapshot().capabilities.len() as i16;
        let max_offset = total.saturating_sub(1).max(0) as u16;
        let next = if delta.is_positive() {
            self.ui.capabilities_scroll.saturating_add(delta as u16)
        } else {
            self.ui
                .capabilities_scroll
                .saturating_sub(delta.unsigned_abs())
        };
        self.ui.capabilities_scroll = next.min(max_offset);
    }
}
