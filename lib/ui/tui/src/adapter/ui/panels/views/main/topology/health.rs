use ratatui::layout::Rect;
use ratatui::prelude::Frame;
use ratatui::text::Line;
use ratatui::widgets::{Paragraph, Wrap};

use crate::app::App;
use crate::panels::views::main::shared::section_title;
use crate::util::collect_problems;

pub fn render_topology_health(frame: &mut Frame, area: Rect, app: &mut App) {
    let snapshot = app.runtime.snapshot();
    let mut lines = Vec::new();
    lines.push(section_title("Health"));
    lines.push(Line::from(format!("Status: {}", snapshot.health.as_str())));
    lines.push(Line::from(""));
    lines.push(section_title("Problems"));
    let problems = collect_problems(app);
    if problems.is_empty() {
        lines.push(Line::from("No problems detected."));
    } else {
        for problem in problems {
            lines.push(Line::from(format!("- {problem}")));
        }
    }
    let paragraph = Paragraph::new(lines).wrap(Wrap { trim: true });
    frame.render_widget(paragraph, area);
}
