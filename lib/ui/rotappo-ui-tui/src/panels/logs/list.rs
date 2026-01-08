//! Log list rendering.

use ratatui::{
    layout::{Margin, Rect},
    prelude::Frame,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{List, ListItem, Scrollbar, ScrollbarOrientation, ScrollbarState},
};

use rotappo_domain::EventLevel;
use crate::app::App;
use crate::state::HoverPanel;
use crate::util::{format_age, traveling_glow};

/// Render the log list panel.
///
/// # Examples
/// ```rust,no_run
/// use ratatui::{backend::TestBackend, Terminal};
/// use rotappo_ui_tui::app::App;
/// use rotappo_ui_tui::panels::render_logs;
///
/// # fn app() -> App { todo!() }
/// let backend = TestBackend::new(80, 24);
/// let mut terminal = Terminal::new(backend).unwrap();
/// let mut app = app();
/// terminal
///     .draw(|frame| render_logs(frame, frame.area(), &mut app))
///     .unwrap();
/// ```
pub fn render_logs(frame: &mut Frame, area: Rect, app: &mut App) {
    app.ui.logs_area = area;
    let hovered = app.ui.hover_panel == HoverPanel::Logs;
    if app.ui.collapsed_logs {
        let block = crate::ui_panel_block!(
            format!("Logs ({})", app.ui.log_config.filter.as_str()),
            hovered
        );
        frame.render_widget(block, area);
        return;
    }
    let events = app.filtered_events();
    let max_items = area.height.saturating_sub(2) as usize;
    let total = events.len();
    let max_offset = total.saturating_sub(max_items);
    let offset = app.ui.log_scroll.min(max_offset as u16) as usize;
    let start = total.saturating_sub(max_items).saturating_sub(offset);

    let items: Vec<ListItem> = events[start..]
        .iter()
        .enumerate()
        .map(|(offset, event)| {
            let level_style = match event.level {
                EventLevel::Info => Style::default().fg(Color::Cyan),
                EventLevel::Warn => Style::default().fg(Color::Yellow),
                EventLevel::Error => Style::default().fg(Color::Red),
            };
            let timestamp = format_age(event.timestamp_ms);
            let line = Line::from(vec![
                Span::styled(format!("[{:<4}]", event.level.as_str()), level_style),
                Span::raw(" "),
                Span::raw(timestamp),
                Span::raw(" "),
                Span::raw(&event.message),
            ]);
            let line_index = start.saturating_add(offset);
            let mut line_style = Style::default();
            if let Some(color) = traveling_glow(line_index, total) {
                line_style = line_style.fg(color);
            }
            ListItem::new(line).style(line_style)
        })
        .collect();

    let updated_secs = app.ui.last_log_emit.elapsed().as_secs();
    let title_left =
        Line::from(format!("Logs ({})", app.ui.log_config.filter.as_str())).left_aligned();
    let title_right =
        Line::from(format!("events: {} | updated: {}s", total, updated_secs)).right_aligned();
    let mut list_block =
        crate::ui_panel_block!(title_left, hovered, app.refresh_pulse_active());
    list_block = list_block.title(title_right);
    let mut list = List::new(items).block(list_block);
    if hovered {
        let active_style = Style::default().bg(Color::Rgb(0, 90, 90));
        list = list.style(active_style);
    }
    frame.render_widget(list, area);

    if total > max_items && max_items > 0 {
        let mut state = ScrollbarState::new(total).position(app.ui.log_scroll as usize);
        let bar =
            Scrollbar::new(ScrollbarOrientation::VerticalRight).style(Style::default().fg(Color::Cyan));
        frame.render_stateful_widget(
            bar,
            area.inner(Margin {
                horizontal: 0,
                vertical: 1,
            }),
            &mut state,
        );
    }
}
