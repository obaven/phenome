//! Tooltip placement helpers.

use ratatui::layout::Rect;
use ratatui::text::Line;

fn line_width(line: &Line) -> u16 {
    line.spans
        .iter()
        .map(|span| span.content.len() as u16)
        .sum()
}

fn tooltip_size(area: Rect, lines: &[Line], max_width_pct: u16, max_height_pct: u16) -> (u16, u16) {
    let max_line = lines.iter().map(line_width).max().unwrap_or(0);
    let mut width = max_line.saturating_add(4);
    let mut height = (lines.len() as u16).saturating_add(2);
    let max_width = area.width.saturating_mul(max_width_pct) / 100;
    let max_height = area.height.saturating_mul(max_height_pct) / 100;
    width = width.min(max_width.max(12));
    height = height.min(max_height.max(6));
    (width, height)
}

/// Position a tooltip near the mouse cursor.
pub fn tooltip_rect_for_mouse(
    area: Rect,
    mouse_pos: Option<(u16, u16)>,
    lines: &[Line],
    max_width_pct: u16,
    max_height_pct: u16,
) -> Rect {
    let (mut width, mut height) = tooltip_size(area, lines, max_width_pct, max_height_pct);
    let (mouse_x, mouse_y) = mouse_pos.unwrap_or((
        area.x.saturating_add(area.width / 2),
        area.y.saturating_add(area.height / 2),
    ));
    let mut x = mouse_x.saturating_add(2);
    let mut y = mouse_y.saturating_add(1);
    let right_edge = area.x.saturating_add(area.width);
    let bottom_edge = area.y.saturating_add(area.height);

    if x + width > right_edge {
        x = mouse_x.saturating_sub(width.saturating_add(2));
    }
    if y + height > bottom_edge {
        y = mouse_y.saturating_sub(height.saturating_add(1));
    }

    if x < area.x {
        x = area.x;
    }
    if y < area.y {
        y = area.y;
    }
    if x + width > right_edge {
        width = right_edge.saturating_sub(x);
    }
    if y + height > bottom_edge {
        height = bottom_edge.saturating_sub(y);
    }

    Rect::new(x, y, width, height)
}

/// Position a tooltip in the bottom-right corner of the area.
pub fn tooltip_rect_in_corner(
    area: Rect,
    lines: &[Line],
    max_width_pct: u16,
    max_height_pct: u16,
    margin_x: u16,
    margin_y: u16,
) -> Rect {
    let (width, height) = tooltip_size(area, lines, max_width_pct, max_height_pct);
    let right_edge = area.x.saturating_add(area.width);
    let bottom_edge = area.y.saturating_add(area.height);
    let x = right_edge.saturating_sub(width.saturating_add(margin_x).max(1));
    let y = bottom_edge.saturating_sub(height.saturating_add(margin_y).max(1));
    Rect::new(x.max(area.x), y.max(area.y), width, height)
}
