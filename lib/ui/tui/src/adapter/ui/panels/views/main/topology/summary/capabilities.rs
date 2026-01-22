use ratatui::layout::Rect;
use ratatui::prelude::Frame;
use ratatui::text::Line;
use ratatui::widgets::{Paragraph, Wrap};

use crate::app::App;
use crate::panels::views::main::shared::section_title;
use crate::util::capability_icon;

pub fn render_topology_capabilities(frame: &mut Frame, area: Rect, app: &mut App) {
    app.ui.capabilities_area = area;
    app.ui.collapsed_capabilities = false;
    let snapshot = app.runtime.snapshot();
    let mut lines = Vec::new();
    lines.push(section_title("Capabilities"));
    if snapshot.capabilities.is_empty() {
        lines.push(Line::from("No capabilities available."));
    } else {
        for capability in &snapshot.capabilities {
            let icon = capability_icon(capability.status);
            lines.push(Line::from(format!(
                "[{icon}] {} ({})",
                capability.name,
                capability.status.as_str()
            )));
        }
    }
    let paragraph = Paragraph::new(lines).wrap(Wrap { trim: true });
    frame.render_widget(paragraph, area);
}
