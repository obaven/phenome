use ratatui::layout::Rect;
use ratatui::prelude::Frame;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

use crate::app::App;

mod assembly;
mod helpers;
mod registry;

pub(super) fn render_detail_sidebar(frame: &mut Frame, area: Rect, app: &mut App) {
    let block = Block::default()
        .title("Details")
        .borders(Borders::TOP)
        .style(Style::default().bg(Color::Rgb(18, 20, 24)).fg(Color::White));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    if let Some(node) = app.graph.selected_node() {
        if let Some(spec_name) = node.id.strip_prefix("reg:") {
            registry::render_registry_detail(frame, inner, app, spec_name);
            return;
        }

        let snapshot = app.runtime.snapshot();
        if let Some(step) = snapshot.assembly_steps.iter().find(|s| s.id == node.id) {
            assembly::render_assembly_detail(frame, inner, app, step);
            return;
        }

        let mut lines = Vec::new();
        lines.push(Line::from(vec![
            Span::raw("Node: "),
            Span::styled(
                &node.id,
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .fg(Color::Yellow),
            ),
        ]));
        lines.push(Line::from(" (No assembly step details found)"));
        let paragraph = Paragraph::new(lines)
            .wrap(Wrap { trim: true })
            .scroll((app.ui.detail_scroll, 0));
        frame.render_widget(paragraph, inner);
        return;
    }

    let mut lines = Vec::new();
    lines.push(Line::from("No node selected."));
    lines.push(Line::from(""));
    lines.push(Line::from("Navigation:"));
    lines.push(Line::from(" [Arrows]: Pan Graph"));
    lines.push(Line::from(" [Click]: Select Node"));
    lines.push(Line::from(" [Enter]: Toggle Panel"));
    lines.push(Line::from(" [Shift+Up/Down]: Scroll This Panel"));
    let paragraph = Paragraph::new(lines)
        .wrap(Wrap { trim: true })
        .scroll((app.ui.detail_scroll, 0));
    frame.render_widget(paragraph, inner);
}
