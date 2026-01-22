use ratatui::layout::Rect;
use ratatui::prelude::Frame;
use ratatui::text::Line;
use ratatui::widgets::{Paragraph, Wrap};

use crate::app::App;
use crate::panels::views::main::shared::section_title;
use phenome_ui_presentation::formatting;

pub fn render_topology_domains(frame: &mut Frame, area: Rect, app: &mut App) {
    let snapshot = app.runtime.snapshot();
    let mut lines = Vec::new();
    lines.push(section_title("Domains"));
    let groups = formatting::assembly_groups(snapshot);
    if groups.is_empty() {
        lines.push(Line::from("No domain data available."));
    } else {
        for group in groups {
            lines.push(Line::from(format!(
                "- {} ({})",
                group.domain.as_str(),
                group.steps.len()
            )));
        }
    }
    let paragraph = Paragraph::new(lines).wrap(Wrap { trim: true });
    frame.render_widget(paragraph, area);
}
