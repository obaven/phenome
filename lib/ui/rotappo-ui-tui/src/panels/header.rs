use ratatui::{
    layout::Rect,
    prelude::{Alignment, Frame},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

use crate::app::App;

/// Render the header status panel.
///
/// # Examples
/// ```rust,no_run
/// use ratatui::{backend::TestBackend, Terminal};
/// use rotappo_ui_tui::app::App;
/// use rotappo_ui_tui::panels::render_header;
///
/// # fn app() -> App { todo!() }
/// let backend = TestBackend::new(80, 24);
/// let mut terminal = Terminal::new(backend).unwrap();
/// let mut app = app();
/// terminal
///     .draw(|frame| render_header(frame, frame.area(), &mut app))
///     .unwrap();
/// ```
pub fn render_header(frame: &mut Frame, area: Rect, app: &mut App) {
    let snapshot = app.runtime.snapshot();
    let watch_label = if app.ui.auto_refresh { "watch:on" } else { "watch:off" };
    let watch_style = if app.ui.auto_refresh {
        Style::default().fg(Color::Green)
    } else {
        Style::default().fg(Color::Yellow)
    };
    let (action_label, action_style) =
        if let (Some(action), Some(status)) = (snapshot.last_action, snapshot.last_action_status) {
            let label = format!("action:{} {}", action, status.as_str());
            let style = match status {
                rotappo_domain::ActionStatus::Running => Style::default().fg(Color::Yellow),
                rotappo_domain::ActionStatus::Failed => Style::default().fg(Color::Red),
                rotappo_domain::ActionStatus::Succeeded => Style::default().fg(Color::Green),
                rotappo_domain::ActionStatus::Pending => Style::default().fg(Color::Gray),
            };
            (label, style)
        } else {
            ("action:idle".to_string(), Style::default().fg(Color::Gray))
        };

    let line = Line::from(vec![
        Span::styled("Rotappo TUI", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(" | host: "),
        Span::styled(
            app.context.host_domain.as_str(),
            Style::default().fg(Color::Cyan),
        ),
        Span::raw(" | config: "),
        Span::raw(app.context.config_path.display().to_string()),
        Span::raw(" | "),
        Span::styled(watch_label, watch_style),
        Span::raw(" | "),
        Span::styled(action_label, action_style),
    ]);

    let mut block = Block::default().borders(Borders::ALL).title("Status");
    if app.refresh_pulse_active() {
        block = block.style(Style::default().fg(Color::Cyan));
    }
    let paragraph = Paragraph::new(line).block(block).alignment(Alignment::Left);
    frame.render_widget(paragraph, area);

    let gear_width = 5u16.min(area.width);
    let gear_height = area.height;
    let x = area
        .x
        .saturating_add(area.width.saturating_sub(gear_width));
    let gear_area = Rect::new(x, area.y, gear_width, gear_height);
    app.ui.settings_gear_area = gear_area;
    let gear_style = if app.ui.collapsed_settings {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::Black).bg(Color::Cyan)
    };
    let gear = Paragraph::new("⚙")
        .block(Block::default().borders(Borders::ALL))
        .alignment(Alignment::Center)
        .style(gear_style);
    frame.render_widget(gear, gear_area);
}
