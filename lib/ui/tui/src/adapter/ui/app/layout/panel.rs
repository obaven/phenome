//! Panel identifiers used for layout, hover, and collapse logic.

/// Panels displayed in the TUI shell.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PanelId {
    Help,
    Notifications,
}
