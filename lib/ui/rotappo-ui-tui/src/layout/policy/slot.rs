//! Slot-specific policy configuration.

use super::PanelPriority;

/// Per-slot overrides for visibility, collapse state, and grid position.
#[derive(Clone, Debug, Default)]
pub struct SlotOverride {
    pub visible: Option<bool>,
    pub collapsed: Option<bool>,
    pub row: Option<usize>,
    pub col: Option<usize>,
    pub row_span: Option<usize>,
    pub col_span: Option<usize>,
}

/// Policy metadata for a layout slot.
///
/// # Examples
/// ```rust
/// use rotappo_ui_tui::layout::{PanelPriority, SlotPolicy};
///
/// let policy = SlotPolicy::new(PanelPriority::High).min_size(4, 3);
/// assert_eq!(policy.min_width, Some(4));
/// ```
#[derive(Clone, Debug)]
pub struct SlotPolicy {
    pub priority: PanelPriority,
    pub movable: bool,
    pub min_width: Option<u16>,
    pub min_height: Option<u16>,
    pub max_width: Option<u16>,
    pub max_height: Option<u16>,
}

impl SlotPolicy {
    /// Create a new policy with the provided priority.
    pub fn new(priority: PanelPriority) -> Self {
        Self {
            priority,
            movable: false,
            min_width: None,
            min_height: None,
            max_width: None,
            max_height: None,
        }
    }

    /// Toggle whether the slot can be moved via offsets.
    pub fn movable(mut self, value: bool) -> Self {
        self.movable = value;
        self
    }

    /// Configure the minimum slot size.
    pub fn min_size(mut self, width: u16, height: u16) -> Self {
        self.min_width = Some(width);
        self.min_height = Some(height);
        self
    }

    /// Configure the maximum slot size.
    pub fn max_size(mut self, width: u16, height: u16) -> Self {
        self.max_width = Some(width);
        self.max_height = Some(height);
        self
    }
}
