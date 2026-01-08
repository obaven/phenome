//! Grid resolution utilities.

use std::collections::HashMap;
use std::time::Instant;

use ratatui::layout::{Constraint, Direction, Layout, Rect};

use super::{GridLayout, GridSpec, GridSlot, TrackSize};

/// Resolves a grid spec into rectangles for the given area.
pub struct GridResolver;

impl GridResolver {
    /// Resolve a grid spec into concrete rectangles.
    ///
    /// # Examples
    /// ```rust
    /// use ratatui::layout::Rect;
    /// use rotappo_ui_tui::layout::{GridResolver, GridSpec, GridSlot, TrackSize};
    ///
    /// let spec = GridSpec::new(
    ///     vec![TrackSize::Percent(50), TrackSize::Percent(50)],
    ///     vec![TrackSize::Percent(100)],
    /// )
    /// .with_slots(vec![GridSlot::new("top", 0, 0), GridSlot::new("bottom", 1, 0)]);
    /// let layout = GridResolver::resolve(Rect::new(0, 0, 10, 10), &spec);
    /// assert_eq!(layout.rect("top").unwrap().height, 5);
    /// ```
    pub fn resolve(area: Rect, spec: &GridSpec) -> GridLayout {
        let rows = resolve_tracks(area, Direction::Vertical, &spec.rows);
        let cols = resolve_tracks(area, Direction::Horizontal, &spec.cols);
        let mut rects = HashMap::new();

        for slot in &spec.slots {
            if !slot.visible {
                continue;
            }
            if let Some(rect) = slot_rect(area, slot, &rows, &cols) {
                rects.insert(slot.id.clone(), rect);
            }
        }

        GridLayout {
            area,
            rects,
            resolved_at: Instant::now(),
        }
    }
}

fn resolve_tracks(area: Rect, direction: Direction, tracks: &[TrackSize]) -> Vec<Rect> {
    if tracks.is_empty() {
        return Vec::new();
    }
    let constraints: Vec<Constraint> = tracks.iter().map(|track| track.to_constraint()).collect();
    Layout::default()
        .direction(direction)
        .constraints(constraints)
        .split(area)
        .to_vec()
}

fn slot_rect(area: Rect, slot: &GridSlot, rows: &[Rect], cols: &[Rect]) -> Option<Rect> {
    if slot.row >= rows.len() || slot.col >= cols.len() {
        return None;
    }
    let row_end = slot.row.saturating_add(slot.row_span).min(rows.len());
    let col_end = slot.col.saturating_add(slot.col_span).min(cols.len());
    if row_end == slot.row || col_end == slot.col {
        return None;
    }
    let top = rows[slot.row].y;
    let left = cols[slot.col].x;
    let bottom = rows[row_end - 1]
        .y
        .saturating_add(rows[row_end - 1].height);
    let right = cols[col_end - 1]
        .x
        .saturating_add(cols[col_end - 1].width);
    let cell_width = right.saturating_sub(left);
    let cell_height = bottom.saturating_sub(top);
    let mut width = cell_width;
    let mut height = cell_height;
    if let Some(max_width) = slot.max_width {
        width = width.min(max_width);
    }
    if let Some(max_height) = slot.max_height {
        height = height.min(max_height);
    }
    if let Some(min_width) = slot.min_width {
        width = width.max(min_width).min(cell_width);
    }
    if let Some(min_height) = slot.min_height {
        height = height.max(min_height).min(cell_height);
    }
    let mut x = left;
    let mut y = top;
    if slot.movable && (slot.offset_x != 0 || slot.offset_y != 0) {
        let mut new_x = (x as i32).saturating_add(slot.offset_x as i32);
        let mut new_y = (y as i32).saturating_add(slot.offset_y as i32);
        let min_x = area.x as i32;
        let min_y = area.y as i32;
        let max_x = area
            .x
            .saturating_add(area.width)
            .saturating_sub(width) as i32;
        let max_y = area
            .y
            .saturating_add(area.height)
            .saturating_sub(height) as i32;
        if new_x < min_x {
            new_x = min_x;
        }
        if new_x > max_x {
            new_x = max_x;
        }
        if new_y < min_y {
            new_y = min_y;
        }
        if new_y > max_y {
            new_y = max_y;
        }
        x = new_x.max(0) as u16;
        y = new_y.max(0) as u16;
    }
    Some(Rect::new(x, y, width, height))
}
