use ratatui::layout::Rect;
use ratatui::prelude::Frame;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Wrap};

use crate::app::App;
use phenome_domain::AssemblyStep;

pub(super) fn render_metadata(frame: &mut Frame, area: Rect, app: &App, step: &AssemblyStep) {
    let mut lines = Vec::new();
    lines.push(Line::from(vec![Span::styled(
        "Metadata",
        Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD),
    )]));
    lines.push(Line::from(format!(" ID:     {}", step.id)));
    lines.push(Line::from(format!(" Status: {}", step.status.as_str())));
    lines.push(Line::from(format!(" Domain: {}", step.domain)));
    lines.push(Line::from(format!(" Kind:   {}", step.kind)));
    if let Some(pod) = &step.pod {
        lines.push(Line::from(format!(" Pod:    {pod}")));
    }

    let paragraph = Paragraph::new(lines)
        .wrap(Wrap { trim: true })
        .scroll((app.ui.detail_scroll, 0));
    frame.render_widget(paragraph, area);
}
