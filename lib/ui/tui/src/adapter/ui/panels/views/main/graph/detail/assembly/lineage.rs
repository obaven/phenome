use ratatui::layout::Rect;
use ratatui::prelude::Frame;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

use phenome_domain::{AssemblyStep, Snapshot};

pub(super) fn render_lineage(
    frame: &mut Frame,
    area: Rect,
    snapshot: &Snapshot,
    step: &AssemblyStep,
) {
    let mut lineage_spans = Vec::new();
    lineage_spans.push(Span::styled(
        "Flow: ",
        Style::default()
            .fg(Color::Magenta)
            .add_modifier(Modifier::BOLD),
    ));

    if !step.depends_on.is_empty() {
        lineage_spans.push(Span::styled(
            format!("[{}]", step.depends_on.join(", ")),
            Style::default().fg(Color::Gray),
        ));
        lineage_spans.push(Span::raw(" -> "));
    } else {
        lineage_spans.push(Span::raw("(Root) -> "));
    }

    lineage_spans.push(Span::styled(
        format!("({})", step.id),
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
    ));
    lineage_spans.push(Span::raw(" -> "));

    let children: Vec<String> = snapshot
        .assembly_steps
        .iter()
        .filter(|s| s.depends_on.contains(&step.id))
        .map(|s| s.id.clone())
        .collect();

    if !children.is_empty() {
        lineage_spans.push(Span::styled(
            format!("[{}]", children.join(", ")),
            Style::default().fg(Color::Cyan),
        ));
    } else {
        lineage_spans.push(Span::raw("(Leaf)"));
    }

    frame.render_widget(Paragraph::new(Line::from(lineage_spans)), area);
}
