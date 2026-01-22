//! Hover context tracking.

/// Panel currently under the mouse cursor.
///
/// # Examples
/// ```rust
/// use phenome_ui_tui::state::HoverPanel;
///
/// assert_eq!(HoverPanel::None, HoverPanel::None);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HoverPanel {
    None,
    Assembly,
    Graph,
    Capabilities,
    Actions,
    Logs,
}
