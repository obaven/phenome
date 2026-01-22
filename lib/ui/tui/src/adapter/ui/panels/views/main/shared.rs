use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};

use crate::app::App;

pub(super) fn section_title(label: &str) -> Line<'_> {
    Line::from(Span::styled(
        label.to_string(),
        Style::default()
            .fg(Color::LightBlue)
            .add_modifier(Modifier::BOLD),
    ))
}

pub(super) fn reset_panel_areas(app: &mut App) {
    app.ui.actions_area = Rect::default();
    app.ui.assembly_area = Rect::default();
    app.ui.capabilities_area = Rect::default();
    app.ui.logs_area = Rect::default();
    app.ui.collapsed_actions = true;
    app.ui.collapsed_logs = true;
    app.ui.collapsed_assembly_steps = true;
    app.ui.collapsed_capabilities = true;
}
