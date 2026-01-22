//! Layout-driven helpers for panel sizing and visibility.

use crate::app::{App, PanelId};

impl App {
    /// Resolve whether a panel is collapsed from the UI state.
    pub fn panel_collapsed(&self, panel: PanelId) -> bool {
        self.is_collapsed(panel)
    }
}
