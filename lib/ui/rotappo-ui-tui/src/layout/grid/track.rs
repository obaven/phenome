//! Track sizing options for grid layouts.

use ratatui::layout::Constraint;

/// Size definition for grid rows or columns.
///
/// # Examples
/// ```rust
/// use rotappo_ui_tui::layout::TrackSize;
///
/// let size = TrackSize::Percent(50);
/// assert_eq!(size, TrackSize::Percent(50));
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TrackSize {
    Fixed(u16),
    Percent(u16),
    Min(u16),
    Max(u16),
    Fill(u16),
}

impl TrackSize {
    pub(crate) fn to_constraint(self) -> Constraint {
        match self {
            TrackSize::Fixed(size) => Constraint::Length(size),
            TrackSize::Percent(value) => Constraint::Percentage(value),
            TrackSize::Min(size) => Constraint::Min(size),
            TrackSize::Max(size) => Constraint::Max(size),
            TrackSize::Fill(weight) => Constraint::Fill(weight),
        }
    }
}
