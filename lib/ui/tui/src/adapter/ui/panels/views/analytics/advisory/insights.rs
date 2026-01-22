use ratatui::{
    layout::Rect,
    prelude::Frame,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Paragraph, Wrap},
};

use crate::app::App;

pub fn render_insights(frame: &mut Frame, area: Rect, app: &mut App) {
    let mut lines = Vec::new();
    lines.push(section_title("Insights"));

    match app.analytics_anomalies.as_ref() {
        Some(anomalies) if !anomalies.is_empty() => {
            for anomaly in anomalies.iter().take(8) {
                lines.push(Line::from(format!(
                    "- [{}] {}",
                    format!("{:?}", anomaly.severity).to_lowercase(),
                    anomaly.description
                )));
            }
        }
        _ => {
            lines.push(Line::from("No anomalies detected."));
        }
    }

    let paragraph = Paragraph::new(lines).wrap(Wrap { trim: true });
    frame.render_widget(paragraph, area);
}

fn section_title(label: &'static str) -> Line<'static> {
    Line::from(Span::styled(
        label,
        Style::default().fg(Color::LightBlue).add_modifier(Modifier::BOLD),
    ))
}
