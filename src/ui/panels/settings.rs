use ratatui::{
    layout::Rect,
    prelude::{Alignment, Frame},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
};

use crate::ui::app::App;
use crate::ui::state::HoverPanel;

pub fn render_settings(frame: &mut Frame, area: Rect, app: &mut App) {
    app.ui.settings_area = area;
    if app.ui.collapsed_settings {
        app.ui.settings_controls_row = None;
        let mut block = Block::default().title("Settings").borders(Borders::ALL);
        if app.ui.hover_panel == HoverPanel::Settings {
            block = block.style(Style::default().bg(Color::Rgb(0, 90, 90)));
        }
        frame.render_widget(block, area);
        return;
    }

    let view = SettingsView {
        host: app.backend.config.network.host_domain.to_string(),
        plan_path: app.backend.plan_path.display().to_string(),
        config_path: app.backend.config_path.display().to_string(),
        log_filter: app.ui.log_config.filter.as_str().to_string(),
        log_interval: app.ui.log_config.interval.as_secs(),
        log_paused: app.ui.log_paused,
        auto_refresh: app.ui.auto_refresh,
        selected: app.ui.settings_selected,
    };
    let lines = settings_lines(&view);
    let controls_row = area
        .inner(ratatui::layout::Margin {
            horizontal: 1,
            vertical: 1,
        })
        .y
        .checked_add(lines.len().saturating_sub(1) as u16);
    app.ui.settings_controls_row = controls_row;
    let mut block = Block::default().title("Settings").borders(Borders::ALL);
    if app.ui.hover_panel == HoverPanel::Settings {
        block = block.style(Style::default().bg(Color::Rgb(0, 90, 90)));
    }
    let mut panel = Paragraph::new(lines)
        .block(block)
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: true });
    if app.ui.hover_panel == HoverPanel::Settings {
        panel = panel.style(Style::default().bg(Color::Rgb(0, 90, 90)));
    }
    frame.render_widget(panel, area);
}

struct SettingsView {
    host: String,
    plan_path: String,
    config_path: String,
    log_filter: String,
    log_interval: u64,
    log_paused: bool,
    auto_refresh: bool,
    selected: usize,
}

fn settings_lines(view: &SettingsView) -> Vec<Line> {
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
        Line::from(format!("Plan: {}", view.plan_path)),
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
