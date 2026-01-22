use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::prelude::Frame;
use ratatui::style::{Color, Modifier, Style, Stylize};
use ratatui::widgets::{Block, Borders, Padding, Paragraph};

use crate::app::App;
use crate::util::centered_rect;

mod cards;
mod format;
mod stats;

pub fn render_realtime(frame: &mut Frame, area: Rect, app: &mut App) {
    let app_metrics = app
        .analytics_metrics
        .as_ref()
        .map(|metrics| metrics.as_slice())
        .unwrap_or_default();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(10),
            Constraint::Min(0),
        ])
        .split(area);

    frame.render_widget(
        Paragraph::new("Real-time Metrics")
            .style(
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )
            .block(Block::default().borders(Borders::BOTTOM)),
        chunks[0],
    );

    if app_metrics.is_empty() {
        frame.render_widget(
            Paragraph::new("Waiting for metrics stream...")
                .style(Style::default().fg(Color::DarkGray).italic())
                .alignment(Alignment::Center),
            centered_rect(50, 50, area),
        );
        return;
    }

    let totals = stats::aggregate_metrics(app_metrics);

    let stat_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[1]);

    let cpu_text = if totals.cpu_valid {
        format!("{:.2} cores", totals.cpu_sum)
    } else {
        "N/A".to_string()
    };
    cards::render_stat_card(
        frame,
        stat_layout[0],
        "Total CPU Load",
        &cpu_text,
        Color::LightGreen,
    );

    let mem_text = if totals.mem_valid {
        format::format_bytes(totals.mem_sum)
    } else {
        "N/A".to_string()
    };
    cards::render_stat_card(
        frame,
        stat_layout[1],
        "Total Memory Usage",
        &mem_text,
        Color::LightMagenta,
    );

    let info = stats::build_info(app_metrics);

    frame.render_widget(
        Paragraph::new(info)
            .style(Style::default().fg(Color::Gray))
            .alignment(Alignment::Center)
            .block(Block::default().padding(Padding::top(1))),
        chunks[2],
    );
}
