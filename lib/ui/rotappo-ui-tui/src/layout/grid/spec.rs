//! Grid specification definitions.

use super::{GridSlot, TrackSize};

/// Grid definition with rows, columns, and slots.
///
/// # Examples
/// ```rust
/// use rotappo_ui_tui::layout::{GridSpec, GridSlot, TrackSize};
///
/// let spec = GridSpec::new(
///     vec![TrackSize::Fixed(2)],
///     vec![TrackSize::Percent(100)],
/// )
/// .with_slots(vec![GridSlot::new("header", 0, 0)]);
/// assert_eq!(spec.rows.len(), 1);
/// ```
#[derive(Clone, Debug)]
pub struct GridSpec {
    pub rows: Vec<TrackSize>,
    pub cols: Vec<TrackSize>,
    pub slots: Vec<GridSlot>,
}

impl GridSpec {
    /// Create an empty grid specification with the given tracks.
    pub fn new(rows: Vec<TrackSize>, cols: Vec<TrackSize>) -> Self {
        Self {
            rows,
            cols,
            slots: Vec::new(),
        }
    }

    /// Populate the spec with an explicit slot list.
    pub fn with_slots(mut self, slots: Vec<GridSlot>) -> Self {
        self.slots = slots;
        self
    }
}
