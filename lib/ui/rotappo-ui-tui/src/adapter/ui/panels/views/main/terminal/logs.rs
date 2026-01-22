use ratatui::layout::Rect;
use ratatui::prelude::Frame;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Wrap};

use crate::app::App;
use crate::panels::views::main::shared::section_title;
use phenome_domain::EventLevel;

pub fn render_terminal_logs(frame: &mut Frame, area: Rect, app: &mut App) {
    app.ui.logs_area = area;
    app.ui.collapsed_logs = false;
    let mut lines = Vec::new();
    lines.push(section_title("Stream"));
    lines.push(Line::from(format!(
        "Filter: {}  Interval: {}s  Watch: {}",
        app.ui.log_config.filter.as_str(),
        app.ui.log_config.interval.as_secs(),
        if app.ui.auto_refresh { "on" } else { "off" }
    )));
    lines.push(Line::from(""));

    let events = app.filtered_events();
    if events.is_empty() {
        lines.push(Line::from("No events captured yet."));
    } else {
        for event in events {
            let level_style = match event.level {
                EventLevel::Info => Style::default().fg(Color::Cyan),
                EventLevel::Warn => Style::default().fg(Color::Yellow),
                EventLevel::Error => Style::default().fg(Color::Red),
            };
            lines.push(Line::from(vec![
                Span::styled(event.level.as_str(), level_style),
                Span::raw(" "),
                Span::raw(event.message.as_str()),
            ]));
        }
    }

    let paragraph = Paragraph::new(lines)
        .wrap(Wrap { trim: true })
        .scroll((app.ui.log_scroll, 0));
    frame.render_widget(paragraph, area);
}
