//! Grouping rules for slots.

use crate::layout::SlotId;

/// Grouping policy for a collection of slots.
///
/// # Examples
/// ```rust
/// use rotappo_ui_tui::layout::GroupPolicy;
///
/// let policy = GroupPolicy::new("side", vec!["a".into(), "b".into()]).exclusive(true);
/// assert!(policy.exclusive);
/// ```
#[derive(Clone, Debug)]
pub struct GroupPolicy {
    pub name: String,
    pub slots: Vec<SlotId>,
    pub exclusive: bool,
    pub min_width: u16,
    pub min_height: u16,
}

impl GroupPolicy {
    /// Create a new group with the provided slot ids.
    pub fn new(name: impl Into<String>, slots: Vec<SlotId>) -> Self {
        Self {
            name: name.into(),
            slots,
            exclusive: false,
            min_width: 0,
            min_height: 0,
        }
    }

    /// Toggle exclusivity (only one slot in the group can be open).
    pub fn exclusive(mut self, value: bool) -> Self {
        self.exclusive = value;
        self
    }

    /// Define the minimum available area for the group.
    pub fn min_area(mut self, width: u16, height: u16) -> Self {
        self.min_width = width;
        self.min_height = height;
        self
    }
}
