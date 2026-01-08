//! Settings panel view model.

use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
};

#[derive(Clone)]
pub(super) struct SettingsView {
    pub(super) host: String,
    pub(super) assembly_path: String,
    pub(super) config_path: String,
    pub(super) log_filter: String,
    pub(super) log_interval: u64,
    pub(super) log_paused: bool,
    pub(super) auto_refresh: bool,
    pub(super) selected: usize,
}

pub(super) fn settings_lines(view: &SettingsView) -> Vec<Line> {
    let apply_style = if view.selected == 0 {
        Style::default().fg(Color::Black).bg(Color::Cyan)
    } else {
        Style::default().fg(Color::Cyan)
    };
    let cancel_style = if view.selected == 1 {
        Style::default().fg(Color::Black).bg(Color::Cyan)
    } else {
        Style::default().fg(Color::Cyan)
    };
    vec![
        Line::from(Span::styled(
            "Runtime",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from(format!("Host: {}", view.host)),
        Line::from(format!("Assembly: {}", view.assembly_path)),
        Line::from(format!("Config: {}", view.config_path)),
        Line::from(""),
        Line::from(Span::styled(
            "UI",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from(format!(
            "Log filter: {} (press f to cycle)",
            view.log_filter
        )),
        Line::from(format!("Stream interval: {}s", view.log_interval)),
        Line::from(format!(
            "Stream paused: {} (hold p to toggle)",
            view.log_paused
        )),
        Line::from(format!(
            "Watch mode: {} (press w to toggle)",
            if view.auto_refresh { "on" } else { "off" }
        )),
        Line::from(""),
        Line::from(Span::styled(
            "Controls",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from(vec![
            Span::styled("[Apply]", apply_style),
            Span::raw("  "),
            Span::styled("[Cancel]", cancel_style),
        ]),
    ]
}
