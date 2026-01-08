//! Tooltip overlay rendering.

use ratatui::{
    layout::Rect,
    prelude::Frame,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
};

use crate::app::App;
use crate::util::{tooltip_rect_for_mouse, tooltip_rect_in_corner};

/// Render hover and pinned tooltips.
///
/// # Examples
/// ```rust,no_run
/// use ratatui::{backend::TestBackend, Terminal};
/// use rotappo_ui_tui::app::App;
/// use rotappo_ui_tui::panels::render_tooltip;
///
/// # fn app() -> App { todo!() }
/// let backend = TestBackend::new(80, 24);
/// let mut terminal = Terminal::new(backend).unwrap();
/// let mut app = app();
/// terminal.draw(|frame| render_tooltip(frame, &mut app)).unwrap();
/// ```
pub fn render_tooltip(frame: &mut Frame, app: &mut App) {
    let pinned = app.ui.pinned_tooltip.clone();
    let hover = app.current_tooltip();
    let mut pinned_area = None;

    if let Some(pinned) = pinned {
        let lines = tooltip_lines(&pinned);
        let area = tooltip_rect_in_corner(frame.area(), &lines, 38, 30, 2, 1);
        frame.render_widget(Clear, area);
        render_tooltip_box(frame, area, lines);
        pinned_area = Some(area);
    }

    if let Some(hover) = hover {
        let lines = tooltip_lines(&hover);
        let mut area = tooltip_rect_for_mouse(frame.area(), app.ui.mouse_pos, &lines, 45, 35);
        if let Some(pinned_area) = pinned_area {
            if rects_overlap(area, pinned_area) {
                let alternate_pos = app
                    .ui
                    .mouse_pos
                    .map(|(x, y)| (x.saturating_sub(4), y.saturating_sub(2)));
                area = tooltip_rect_for_mouse(frame.area(), alternate_pos, &lines, 45, 35);
            }
        }
        frame.render_widget(Clear, area);
        render_tooltip_box(frame, area, lines);
    }
}

fn render_tooltip_box(frame: &mut Frame, area: Rect, lines: Vec<Line>) {
    let panel = Paragraph::new(lines)
        .block(Block::default().title("Tooltip").borders(Borders::ALL))
        .wrap(Wrap { trim: true });
    frame.render_widget(panel, area);
}

fn tooltip_lines(tooltip: &crate::state::Tooltip) -> Vec<Line> {
    std::iter::once(Line::from(Span::styled(
        tooltip.title.clone(),
        Style::default().add_modifier(Modifier::BOLD),
    )))
    .chain(tooltip.lines.iter().cloned().map(Line::from))
    .collect()
}

fn rects_overlap(a: Rect, b: Rect) -> bool {
    let a_right = a.x.saturating_add(a.width);
    let a_bottom = a.y.saturating_add(a.height);
    let b_right = b.x.saturating_add(b.width);
    let b_bottom = b.y.saturating_add(b.height);
    a.x < b_right && a_right > b.x && a.y < b_bottom && a_bottom > b.y
}
