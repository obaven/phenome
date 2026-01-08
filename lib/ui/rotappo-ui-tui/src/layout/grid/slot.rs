//! Slot identifiers and configuration for grid layouts.

use std::borrow::Borrow;

/// Stable identifier for grid slots.
///
/// # Examples
/// ```rust
/// use rotappo_ui_tui::layout::SlotId;
///
/// let id = SlotId::new("header");
/// assert_eq!(id.as_str(), "header");
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct SlotId(String);

impl SlotId {
    /// Create a new slot identifier.
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }

    /// Borrow the slot identifier as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for SlotId {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl Borrow<str> for SlotId {
    fn borrow(&self) -> &str {
        &self.0
    }
}

/// Grid slot configuration and sizing hints.
///
/// # Examples
/// ```rust
/// use rotappo_ui_tui::layout::GridSlot;
///
/// let slot = GridSlot::new("body", 0, 0)
///     .span(2, 1)
///     .movable(true)
///     .with_min_size(10, 4);
/// assert_eq!(slot.row_span, 2);
/// assert!(slot.movable);
/// ```
#[derive(Clone, Debug)]
pub struct GridSlot {
    pub id: SlotId,
    pub row: usize,
    pub col: usize,
    pub row_span: usize,
    pub col_span: usize,
    pub visible: bool,
    pub movable: bool,
    pub min_width: Option<u16>,
    pub min_height: Option<u16>,
    pub max_width: Option<u16>,
    pub max_height: Option<u16>,
    pub offset_x: i16,
    pub offset_y: i16,
}

impl GridSlot {
    /// Create a new slot at the given row and column.
    pub fn new(id: impl Into<SlotId>, row: usize, col: usize) -> Self {
        Self {
            id: id.into(),
            row,
            col,
            row_span: 1,
            col_span: 1,
            visible: true,
            movable: false,
            min_width: None,
            min_height: None,
            max_width: None,
            max_height: None,
            offset_x: 0,
            offset_y: 0,
        }
    }

    /// Set the row and column span for the slot.
    pub fn span(mut self, row_span: usize, col_span: usize) -> Self {
        self.row_span = row_span.max(1);
        self.col_span = col_span.max(1);
        self
    }

    /// Mark this slot as hidden.
    pub fn hidden(mut self) -> Self {
        self.visible = false;
        self
    }

    /// Enable or disable slot movement.
    pub fn movable(mut self, value: bool) -> Self {
        self.movable = value;
        self
    }

    /// Configure a minimum width and height.
    pub fn with_min_size(mut self, width: u16, height: u16) -> Self {
        self.min_width = Some(width);
        self.min_height = Some(height);
        self
    }

    /// Configure a maximum width and height.
    pub fn with_max_size(mut self, width: u16, height: u16) -> Self {
        self.max_width = Some(width);
        self.max_height = Some(height);
        self
    }

    /// Apply an offset for movable slots.
    pub fn offset(mut self, x: i16, y: i16) -> Self {
        self.offset_x = x;
        self.offset_y = y;
        self
    }
}
