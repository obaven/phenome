use ratatui::layout::Rect;
use ratatui::prelude::Frame;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Wrap};

use crate::app::App;

use super::ProvisionSets;

pub(super) fn render_capabilities(
    frame: &mut Frame,
    area: Rect,
    app: &App,
    provisions: &ProvisionSets<'_>,
) {
    let mut lines = Vec::new();
    lines.push(Line::from(vec![Span::styled(
        "Capabilities",
        Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
    )]));

    if provisions.other_provs.is_empty() {
        lines.push(Line::from(Span::styled(
            "(None)",
            Style::default().fg(Color::DarkGray),
        )));
    } else {
        for prov in &provisions.other_provs {
            lines.push(Line::from(format!("✨ {prov}")));
        }
    }

    let paragraph = Paragraph::new(lines)
        .wrap(Wrap { trim: true })
        .scroll((app.ui.detail_scroll, 0));
    frame.render_widget(paragraph, area);
}
