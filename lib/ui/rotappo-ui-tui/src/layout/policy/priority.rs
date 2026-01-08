//! Priority levels for panels.

/// Priority used when resolving tight layout scenarios.
///
/// # Examples
/// ```rust
/// use rotappo_ui_tui::layout::PanelPriority;
///
/// assert!(PanelPriority::High.rank() > PanelPriority::Low.rank());
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PanelPriority {
    Critical,
    High,
    Normal,
    Low,
}

impl PanelPriority {
    /// Convert the priority to a numeric rank.
    pub fn rank(self) -> u8 {
        match self {
            PanelPriority::Critical => 4,
            PanelPriority::High => 3,
            PanelPriority::Normal => 2,
            PanelPriority::Low => 1,
        }
    }
}
