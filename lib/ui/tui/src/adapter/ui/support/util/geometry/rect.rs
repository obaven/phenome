//! Rectangle helpers for overlay placement.

use ratatui::layout::{Constraint, Direction, Layout, Rect};

/// Center a rectangle within a parent area.
///
/// # Examples
/// ```rust
/// use ratatui::layout::Rect;
/// use phenome_ui_tui::util::centered_rect;
///
/// let area = Rect::new(0, 0, 100, 50);
/// let centered = centered_rect(50, 50, area);
/// assert!(centered.width <= area.width);
/// ```
pub fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);

    let horizontal = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1]);

    horizontal[1]
}

/// Anchor a rectangle near the bottom-right corner.
///
/// # Examples
/// ```rust
/// use ratatui::layout::Rect;
/// use phenome_ui_tui::util::anchored_rect;
///
/// let area = Rect::new(0, 0, 100, 50);
/// let anchored = anchored_rect(50, 50, area);
/// assert!(anchored.x > 0);
/// ```
pub fn anchored_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let width = area.width.saturating_mul(percent_x) / 100;
    let height = area.height.saturating_mul(percent_y) / 100;
    Rect::new(
        area.width.saturating_sub(width + 1),
        area.height.saturating_sub(height + 1),
        width,
        height,
    )
}

/// Anchor a rectangle with an additional offset.
pub fn anchored_rect_with_offset(
    percent_x: u16,
    percent_y: u16,
    area: Rect,
    offset_x: i16,
    offset_y: i16,
) -> Rect {
    let base = anchored_rect(percent_x, percent_y, area);
    let x = base.x.saturating_add_signed(offset_x);
    let y = base.y.saturating_add_signed(offset_y);
    Rect::new(x, y, base.width, base.height)
}
