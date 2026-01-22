//! Resolved grid layouts.

use std::collections::HashMap;
use std::time::Instant;

use ratatui::layout::Rect;

use crate::adapter::ui::layout::grid::model::slot::SlotId;

/// Resolved rectangles for a grid specification.
///
/// # Examples
/// ```rust
/// use ratatui::layout::Rect;
/// use phenome_ui_tui::layout::{GridLayout, SlotId};
/// use std::collections::HashMap;
///
/// let mut rects = HashMap::new();
/// rects.insert(SlotId::new("header"), Rect::new(0, 0, 10, 2));
/// let layout = GridLayout { area: Rect::new(0, 0, 10, 2), rects, resolved_at: std::time::Instant::now() };
/// assert!(layout.rect("header").is_some());
/// ```
#[derive(Clone, Debug)]
pub struct GridLayout {
    pub area: Rect,
    pub rects: HashMap<SlotId, Rect>,
    pub resolved_at: Instant,
}

impl GridLayout {
    /// Lookup a resolved rectangle by slot id.
    pub fn rect(&self, id: &str) -> Option<Rect> {
        self.rects
            .iter()
            .find(|(key, _)| key.as_str() == id)
            .map(|(_, value)| *value)
    }
}
