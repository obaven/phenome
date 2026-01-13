use ratatui::{
    layout::Rect,
    prelude::Frame,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Paragraph, Wrap},
};

use crate::app::App;

pub fn render_predictions(frame: &mut Frame, area: Rect, _app: &mut App) {
    let mut lines = Vec::new();
    lines.push(section_title("Predictions"));
    lines.push(Line::from(
        "Scaling predictions will appear once the ML service is available.",
    ));
    lines.push(Line::from("Horizons: 1h, 6h, 24h"));

    let paragraph = Paragraph::new(lines).wrap(Wrap { trim: true });
    frame.render_widget(paragraph, area);
}

fn section_title(label: &'static str) -> Line<'static> {
    Line::from(Span::styled(
        label,
        Style::default().fg(Color::LightBlue).add_modifier(Modifier::BOLD),
    ))
}
