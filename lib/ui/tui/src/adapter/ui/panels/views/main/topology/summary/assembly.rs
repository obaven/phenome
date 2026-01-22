use ratatui::layout::Rect;
use ratatui::prelude::Frame;
use ratatui::widgets::{Paragraph, Wrap};

use crate::app::App;
use crate::util::assembly_lines;

pub fn render_topology_assembly(frame: &mut Frame, area: Rect, app: &mut App) {
    app.ui.assembly_area = area;
    app.ui.collapsed_assembly_steps = false;
    let lines = assembly_lines(app.runtime.snapshot())
        .into_iter()
        .map(|entry| entry.line)
        .collect::<Vec<_>>();
    let paragraph = if lines.is_empty() {
        Paragraph::new("No assembly data available.").wrap(Wrap { trim: true })
    } else {
        Paragraph::new(lines).wrap(Wrap { trim: true })
    };
    let paragraph = paragraph.scroll((app.ui.assembly_scroll, 0));
    frame.render_widget(paragraph, area);
}
