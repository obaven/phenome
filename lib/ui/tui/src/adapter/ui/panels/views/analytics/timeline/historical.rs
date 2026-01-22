use ratatui::{
    layout::Rect,
    prelude::Frame,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Paragraph, Wrap},
};

use crate::app::App;

pub fn render_historical(frame: &mut Frame, area: Rect, app: &mut App) {
    let mut lines = Vec::new();
    lines.push(section_title("Historical Metrics"));
    lines.push(Line::from(
        "Time-series charts and CSV export are not yet connected.",
    ));

    let metrics = app
        .analytics_metrics
        .as_ref()
        .map(|metrics| metrics.len())
        .unwrap_or(0);
    lines.push(Line::from(format!("Cached samples: {metrics}")));

    let paragraph = Paragraph::new(lines).wrap(Wrap { trim: true });
    frame.render_widget(paragraph, area);
}

fn section_title(label: &'static str) -> Line<'static> {
    Line::from(Span::styled(
        label,
        Style::default().fg(Color::LightBlue).add_modifier(Modifier::BOLD),
    ))
}
