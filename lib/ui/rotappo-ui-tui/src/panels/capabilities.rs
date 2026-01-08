use ratatui::{
    layout::{Margin, Rect},
    prelude::Frame,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, Wrap},
};

use crate::app::App;
use crate::state::HoverPanel;
use crate::util::{capability_icon, traveling_glow};

/// Render the capabilities panel.
///
/// # Examples
/// ```rust,no_run
/// use ratatui::{backend::TestBackend, Terminal};
/// use rotappo_ui_tui::app::App;
/// use rotappo_ui_tui::panels::render_capabilities;
///
/// # fn app() -> App { todo!() }
/// let backend = TestBackend::new(80, 24);
/// let mut terminal = Terminal::new(backend).unwrap();
/// let mut app = app();
/// terminal
///     .draw(|frame| render_capabilities(frame, frame.area(), &mut app))
///     .unwrap();
/// ```
pub fn render_capabilities(frame: &mut Frame, area: Rect, app: &mut App) {
    app.ui.capabilities_area = area;
    if app.ui.collapsed_capabilities {
        let mut block = Block::default().title("Capabilities").borders(Borders::ALL);
        if app.ui.hover_panel == HoverPanel::Capabilities {
            block = block.style(Style::default().bg(Color::Rgb(0, 90, 90)));
        }
        frame.render_widget(block, area);
        return;
    }
    let snapshot = app.runtime.snapshot();
    let total_caps = snapshot.capabilities.len();
    let lines: Vec<Line> = snapshot
        .capabilities
        .iter()
        .skip(app.ui.capabilities_scroll as usize)
        .enumerate()
        .map(|(offset, capability)| {
            let status_style = match capability.status {
                rotappo_domain::CapabilityStatus::Ready => Style::default().fg(Color::Green),
                rotappo_domain::CapabilityStatus::Degraded => Style::default().fg(Color::Yellow),
                rotappo_domain::CapabilityStatus::Offline => Style::default().fg(Color::Red),
            };
            let line_index = app.ui.capabilities_scroll as usize + offset;
            let mut line_style = Style::default();
            if let Some(color) = traveling_glow(line_index, total_caps) {
                line_style = line_style.fg(color);
            }
            Line::from(vec![
                Span::styled(
                    format!(
                        "[{} {:<8}]",
                        capability_icon(capability.status),
                        capability.status.as_str()
                    ),
                    status_style,
                ),
                Span::raw(" "),
                Span::raw(&capability.name),
            ])
            .style(line_style)
        })
        .collect();

    let mut panel_block = Block::default().title("Capabilities").borders(Borders::ALL);
    if app.refresh_pulse_active() {
        panel_block = panel_block.style(Style::default().fg(Color::Cyan));
    }
    if app.ui.hover_panel == HoverPanel::Capabilities {
        let active_style = Style::default().bg(Color::Rgb(0, 90, 90));
        panel_block = panel_block.style(active_style);
    }
    let mut panel = Paragraph::new(lines)
        .block(panel_block)
        .wrap(Wrap { trim: true });
    if app.ui.hover_panel == HoverPanel::Capabilities {
        let active_style = Style::default().bg(Color::Rgb(0, 90, 90));
        panel = panel.style(active_style);
    }
    frame.render_widget(panel, area);

    let view_height = area.height.saturating_sub(2) as usize;
    if total_caps > view_height && view_height > 0 {
        let mut state =
            ScrollbarState::new(total_caps).position(app.ui.capabilities_scroll as usize);
        let bar =
            Scrollbar::new(ScrollbarOrientation::VerticalRight).style(Style::default().fg(Color::Cyan));
        frame.render_stateful_widget(
            bar,
            area.inner(Margin {
                horizontal: 0,
                vertical: 1,
            }),
            &mut state,
        );
    }
}
