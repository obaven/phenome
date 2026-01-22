use ratatui::layout::Rect;
use ratatui::prelude::Frame;
use ratatui::text::Line;
use ratatui::widgets::{Paragraph, Wrap};

use crate::app::App;
use crate::panels::views::main::shared::section_title;
use crate::util::collect_problems;

pub fn render_terminal_diagnostics(frame: &mut Frame, area: Rect, app: &mut App) {
    let mut lines = Vec::new();
    lines.push(section_title("Diagnostics"));
    let problems = collect_problems(app);
    if problems.is_empty() {
        lines.push(Line::from("No problems detected."));
    } else {
        for problem in problems.iter().take(8) {
            lines.push(Line::from(format!("- {problem}")));
        }
    }
    lines.push(Line::from(""));
    lines.push(section_title("Overlay"));
    if app.panel_collapsed(crate::app::PanelId::Notifications) {
        lines.push(Line::from("Diagnostics overlay: closed (press n)"));
    } else {
        lines.push(Line::from("Diagnostics overlay: open"));
    }
    let paragraph = Paragraph::new(lines).wrap(Wrap { trim: true });
    frame.render_widget(paragraph, area);
}
