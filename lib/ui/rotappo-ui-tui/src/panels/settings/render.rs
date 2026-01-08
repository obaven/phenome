//! Settings panel rendering.

use ratatui::{
    layout::Rect,
    prelude::{Alignment, Frame},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph, Wrap},
};

use crate::app::App;
use crate::state::HoverPanel;

use super::view::{settings_lines, SettingsView};

/// Render the settings panel.
///
/// # Examples
/// ```rust,no_run
/// use ratatui::{backend::TestBackend, Terminal};
/// use rotappo_ui_tui::app::App;
/// use rotappo_ui_tui::panels::render_settings;
///
/// # fn app() -> App { todo!() }
/// let backend = TestBackend::new(80, 24);
/// let mut terminal = Terminal::new(backend).unwrap();
/// let mut app = app();
/// terminal
///     .draw(|frame| render_settings(frame, frame.area(), &mut app))
///     .unwrap();
/// ```
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
        host: app.context.host_domain.to_string(),
        assembly_path: app.context.assembly_path.display().to_string(),
        config_path: app.context.config_path.display().to_string(),
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
