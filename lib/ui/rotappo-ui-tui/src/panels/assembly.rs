use ratatui::{
    layout::Rect,
    prelude::Frame,
    style::{Color, Style},
    text::Line,
    widgets::{Block, Borders, Gauge, Paragraph, Wrap},
};

use crate::app::App;
use crate::util::format_age;

/// Render assembly progress and snapshot panels.
///
/// # Examples
/// ```rust,no_run
/// use ratatui::{backend::TestBackend, Terminal};
/// use rotappo_ui_tui::app::App;
/// use rotappo_ui_tui::panels::render_assembly;
///
/// # fn app() -> App { todo!() }
/// let backend = TestBackend::new(80, 24);
/// let mut terminal = Terminal::new(backend).unwrap();
/// let mut app = app();
/// terminal
///     .draw(|frame| {
///         let area = frame.area();
///         render_assembly(frame, area, area, &mut app);
///     })
///     .unwrap();
/// ```
pub fn render_assembly(
    frame: &mut Frame,
    assembly_progress_area: Rect,
    snapshot_area: Rect,
    app: &mut App,
) {
    let snapshot = app.runtime.snapshot();
    app.ui.assembly_progress_area = assembly_progress_area;
    app.ui.snapshot_area = snapshot_area;

    if app.ui.collapsed_assembly_progress {
        let block = Block::default()
            .title("Assembly Progress")
            .borders(Borders::ALL);
        frame.render_widget(block, assembly_progress_area);
    } else {
        let percent = snapshot.assembly.percent_complete();
        let mut gauge_block = Block::default()
            .title("Assembly Progress")
            .borders(Borders::ALL);
        if app.refresh_pulse_active() {
            gauge_block = gauge_block.style(Style::default().fg(Color::Cyan));
        }
        let gauge = Gauge::default()
            .block(gauge_block)
            .gauge_style(Style::default().fg(Color::Green))
            .percent(percent)
            .label(format!("{}%", percent));
        frame.render_widget(gauge, assembly_progress_area);
    }

    let age = format_age(snapshot.last_updated_ms);
    let lines = vec![
        Line::from(format!(
            "Ready: {}/{}",
            snapshot.assembly.completed, snapshot.assembly.total
        )),
        Line::from(format!("Initializing: {}", snapshot.assembly.in_progress)),
        Line::from(format!("Queued: {}", snapshot.assembly.blocked)),
        Line::from(format!("Waiting: {}", snapshot.assembly.pending)),
        Line::from(format!("Health: {}", snapshot.health.as_str())),
        Line::from(format!("Last update: {}", age)),
        Line::from(format!(
            "Last action: {}",
            snapshot
                .last_action
                .map(|action| action.to_string())
                .unwrap_or_else(|| "none".to_string())
        )),
    ];

    if app.ui.collapsed_snapshot {
        let mut block = Block::default().title("Snapshot").borders(Borders::ALL);
        if app.ui.hover_snapshot {
            let active_style = Style::default().bg(Color::Rgb(0, 90, 90));
            block = block.style(active_style);
        }
        frame.render_widget(block, snapshot_area);
    } else {
        let mut summary_block = Block::default().title("Snapshot").borders(Borders::ALL);
        if app.refresh_pulse_active() {
            summary_block = summary_block.style(Style::default().fg(Color::Cyan));
        }
        if app.ui.hover_snapshot {
            let active_style = Style::default().bg(Color::Rgb(0, 90, 90));
            summary_block = summary_block.style(active_style);
        }
        let mut summary = Paragraph::new(lines)
            .block(summary_block)
            .wrap(Wrap { trim: true });
        if app.ui.hover_snapshot {
            let active_style = Style::default().bg(Color::Rgb(0, 90, 90));
            summary = summary.style(active_style);
        }
        frame.render_widget(summary, snapshot_area);
    }
}
