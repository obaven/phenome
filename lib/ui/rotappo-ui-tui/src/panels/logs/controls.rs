//! Log control panel rendering.

use ratatui::{
    layout::{Margin, Rect},
    prelude::{Alignment, Frame},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
};

use crate::app::App;
use crate::state::HoverPanel;

/// Render the log control panel.
///
/// # Examples
/// ```rust,no_run
/// use ratatui::{backend::TestBackend, Terminal};
/// use rotappo_ui_tui::app::App;
/// use rotappo_ui_tui::panels::render_log_controls;
///
/// # fn app() -> App { todo!() }
/// let backend = TestBackend::new(80, 24);
/// let mut terminal = Terminal::new(backend).unwrap();
/// let mut app = app();
/// terminal
///     .draw(|frame| render_log_controls(frame, frame.area(), &mut app))
///     .unwrap();
/// ```
pub fn render_log_controls(frame: &mut Frame, area: Rect, app: &mut App) {
    app.ui.log_controls_area = area;
    let hovered = app.ui.hover_panel == HoverPanel::Logs;
    if app.ui.collapsed_log_controls {
        app.ui.log_menu_pinned = false;
        app.ui.log_menu_mode = None;
        app.ui.log_menu_area = Rect::default();
        app.ui.log_menu_len = 0;
        app.ui.log_menu_hover_index = None;
        app.ui.log_filter_tag_area = Rect::default();
        app.ui.log_stream_tag_area = Rect::default();
        let block = crate::ui_panel_block!("Log Controls", hovered);
        frame.render_widget(block, area);
        return;
    }
    let status = if app.ui.log_paused { "paused" } else { "streaming" };
    let filter_tag = format!("[{}]", app.ui.log_config.filter.as_str());
    let stream_tag = format!("[{} {}s]", status, app.ui.log_config.interval.as_secs());
    let line = Line::from(vec![
        Span::raw("Filter "),
        Span::styled(
            &filter_tag,
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::UNDERLINED),
        ),
        Span::raw(" | "),
        Span::raw("Stream "),
        Span::styled(
            &stream_tag,
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::UNDERLINED),
        ),
        Span::raw(" | click tags"),
    ]);
    let panel = Paragraph::new(line)
        .block(crate::ui_panel_block!("Log Controls", hovered))
        .alignment(Alignment::Left);
    frame.render_widget(panel, area);

    let inner = area.inner(Margin {
        horizontal: 1,
        vertical: 1,
    });
    let mut cursor = inner.x;
    cursor = cursor.saturating_add("Filter ".len() as u16);
    app.ui.log_filter_tag_area = Rect::new(cursor, inner.y, filter_tag.len() as u16, 1);
    cursor = cursor.saturating_add(filter_tag.len() as u16);
    cursor = cursor.saturating_add(" | ".len() as u16);
    cursor = cursor.saturating_add("Stream ".len() as u16);
    app.ui.log_stream_tag_area = Rect::new(cursor, inner.y, stream_tag.len() as u16, 1);

    super::menu::render_log_menu(frame, app);
}
