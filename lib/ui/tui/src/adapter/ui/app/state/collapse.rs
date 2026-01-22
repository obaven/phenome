//! Manual panel collapse helpers backed by UiState.

use crate::app::{App, PanelId};

impl App {
    pub fn toggle_notifications_panel(&mut self) {
        let next = !self.panel_collapsed(PanelId::Notifications);
        self.set_panel_collapsed(PanelId::Notifications, next);
    }

    pub(crate) fn is_collapsed(&self, panel: PanelId) -> bool {
        match panel {
            PanelId::Help => self.ui.collapsed_help,
            PanelId::Notifications => self.ui.collapsed_notifications,
        }
    }

    fn set_panel_collapsed(&mut self, panel: PanelId, value: bool) {
        match panel {
            PanelId::Help => self.ui.collapsed_help = value,
            PanelId::Notifications => self.ui.collapsed_notifications = value,
        }
    }
}
