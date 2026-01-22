use crate::state::HoverPanel;

use crate::app::App;

mod actions;
mod assembly;
mod logs;

impl App {
    pub fn scroll_active_panel(&mut self, delta: i16) {
        match self.ui.hover_panel {
            HoverPanel::Logs => self.scroll_logs(delta),
            HoverPanel::Assembly => self.scroll_assembly(delta),
            HoverPanel::Capabilities => self.scroll_capabilities(delta),
            HoverPanel::Actions => self.scroll_actions(delta),
            _ => {
                if delta > 0 {
                    self.select_next_action();
                } else if delta < 0 {
                    self.select_previous_action();
                }
            }
        }
    }
}
