use ratatui::layout::Rect;
use ratatui::prelude::Frame;
use ratatui::text::Line;
use ratatui::widgets::{Paragraph, Wrap};

use crate::app::App;
use crate::panels::views::main::shared::section_title;
use phenome_domain::AssemblyStepStatus;
use phenome_ui_presentation::formatting;

pub fn render_topology_queue(frame: &mut Frame, area: Rect, app: &mut App) {
    let snapshot = app.runtime.snapshot();
    let mut lines = Vec::new();
    lines.push(section_title("Queue State"));
    lines.push(Line::from(format!(
        "Ready: {ready}  Running: {running}",
        ready = snapshot.assembly.completed,
        running = snapshot.assembly.in_progress
    )));
    lines.push(Line::from(format!(
        "Blocked: {blocked}  Pending: {pending}",
        blocked = snapshot.assembly.blocked,
        pending = snapshot.assembly.pending
    )));
    lines.push(Line::from(""));
    lines.push(section_title("Blocked Steps"));
    let mut blocked = Vec::new();
    for group in formatting::assembly_groups(snapshot) {
        for step in group.steps {
            if step.step.status == AssemblyStepStatus::Blocked {
                blocked.push(step.step.id);
            }
        }
    }
    if blocked.is_empty() {
        lines.push(Line::from("No blocked steps."));
    } else {
        for id in blocked.into_iter().take(6) {
            lines.push(Line::from(format!("- {id}")));
        }
    }
    let paragraph = Paragraph::new(lines).wrap(Wrap { trim: true });
    frame.render_widget(paragraph, area);
}
