use crate::bootstrap::utils::format_duration;
use bootstrappo::application::timing::compare_runs;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::prelude::{Alignment, Frame};
use ratatui::style::{Color, Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Row, Table};
use rotappo_ports::{AccessUrlInfo, ComponentState, ComponentStatus, PortSet};
use std::collections::HashMap;
use std::time::Duration;

pub fn render(frame: &mut Frame, _area: Rect, ports: &PortSet) {
    let status = ports.bootstrap.bootstrap_status();
    let states = ports.bootstrap.component_states();
    let total = status
        .total_components
        .unwrap_or_else(|| states.len().max(1));
    let completed = states
        .values()
        .filter(|s| s.status == ComponentStatus::Complete)
        .count();
    let failed = states
        .values()
        .filter(|s| s.status == ComponentStatus::Failed)
        .count();
    let deferred = states
        .values()
        .filter(|s| s.status == ComponentStatus::Deferred)
        .count();
    let success_rate = completed as f32 / total as f32 * 100.0;
    let total_duration = status.total_duration.unwrap_or_default();

    let mut overall_text = vec![
        Line::from(Span::styled(
            "Bootstrap Complete!",
            Style::default().fg(Color::Green).bold(),
        )),
        Line::from(""),
        Line::from(format!(
            "Total Time: {total_time}",
            total_time = format_duration(total_duration)
        )),
        Line::from(format!("Complete: {completed}/{total}")),
        Line::from(format!("Deferred: {deferred}  Failed: {failed}")),
        Line::from(format!("Success Rate: {success_rate:.1}%")),
    ];
    if let Some(line) = build_comparison_line(ports) {
        overall_text.push(Line::from(""));
        overall_text.push(line);
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(8),
            Constraint::Length(6),
            Constraint::Length(8),
            Constraint::Min(0),
        ])
        .split(frame.area());

    let overall = Paragraph::new(overall_text)
        .block(
            Block::default()
                .title("Overall Status")
                .borders(Borders::ALL),
        )
        .alignment(Alignment::Left);
    frame.render_widget(overall, chunks[0]);

    let timing_rows = build_timing_rows(&states);
    let timing_table = Table::new(
        timing_rows,
        [
            Constraint::Percentage(35),
            Constraint::Percentage(25),
            Constraint::Percentage(40),
        ],
    )
    .block(
        Block::default()
            .title("Timing Breakdown")
            .borders(Borders::ALL),
    );
    frame.render_widget(timing_table, chunks[1]);

    let access_rows = build_access_rows(&ports.bootstrap.access_urls());
    let access_table = Table::new(
        access_rows,
        [
            Constraint::Percentage(30),
            Constraint::Percentage(50),
            Constraint::Percentage(20),
        ],
    )
    .block(Block::default().title("Access URLs").borders(Borders::ALL));
    frame.render_widget(access_table, chunks[2]);

    let hotspot_rows = build_hotspot_rows(&states);
    let hotspot_table = Table::new(
        hotspot_rows,
        [
            Constraint::Percentage(50),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
        ],
    )
    .block(Block::default().title("Top Hotspots").borders(Borders::ALL));
    frame.render_widget(hotspot_table, chunks[3]);
}

fn build_comparison_line(ports: &PortSet) -> Option<Line<'static>> {
    let history = ports.bootstrap.timing_history()?;
    if history.entries.len() < 2 {
        return None;
    }
    let mut prior = history.clone();
    let current = prior.entries.pop()?;
    let comparison = compare_runs(&current, &prior);
    let delta = comparison.delta?;
    let label = if delta < 0 { "faster" } else { "slower" };
    let percent = comparison.improvement_percentage.unwrap_or_default().abs();
    Some(Line::from(format!(
        "Comparison: {percent:.1}% {label} ({delta}s vs previous)",
        delta = delta.abs()
    )))
}

fn build_access_rows(urls: &[AccessUrlInfo]) -> Vec<Row<'static>> {
    let mut rows = Vec::new();
    rows.push(Row::new(vec![
        "Service".to_string(),
        "URL".to_string(),
        "Status".to_string(),
    ]));

    if urls.is_empty() {
        rows.push(Row::new(vec![
            "No access URLs discovered".to_string(),
            "-".to_string(),
            "-".to_string(),
        ]));
        return rows;
    }

    for info in urls {
        rows.push(Row::new(vec![
            info.service.clone(),
            info.url.clone(),
            info.status.label().to_string(),
        ]));
    }
    rows
}

fn build_timing_rows(states: &HashMap<String, ComponentState>) -> Vec<Row<'static>> {
    let mut render = Duration::ZERO;
    let mut apply = Duration::ZERO;
    let mut wait = Duration::ZERO;
    for state in states.values() {
        render += state.timing.render_duration.unwrap_or_default();
        apply += state.timing.apply_duration.unwrap_or_default();
        wait += state.timing.wait_duration.unwrap_or_default();
    }

    vec![
        Row::new(vec![
            "Phase".to_string(),
            "Duration".to_string(),
            "Notes".to_string(),
        ]),
        Row::new(vec![
            "Render".to_string(),
            format_duration(render),
            if render == Duration::ZERO {
                "n/a".to_string()
            } else {
                "".to_string()
            },
        ]),
        Row::new(vec![
            "Apply".to_string(),
            format_duration(apply),
            if apply == Duration::ZERO {
                "n/a".to_string()
            } else {
                "".to_string()
            },
        ]),
        Row::new(vec![
            "Wait".to_string(),
            format_duration(wait),
            if wait == Duration::ZERO {
                "n/a".to_string()
            } else {
                "".to_string()
            },
        ]),
    ]
}

fn build_hotspot_rows(states: &HashMap<String, ComponentState>) -> Vec<Row<'static>> {
    let mut durations: Vec<_> = states
        .values()
        .filter_map(|state| state.timing.total_duration.map(|d| (state.id.clone(), d)))
        .collect();
    durations.sort_by_key(|(_, duration)| std::cmp::Reverse(*duration));
    durations.truncate(5);

    let mut rows = Vec::new();
    rows.push(Row::new(vec![
        "Component".to_string(),
        "Total Time".to_string(),
        "Wait".to_string(),
    ]));
    for (id, duration) in durations {
        rows.push(Row::new(vec![
            id,
            format_duration(duration),
            "-".to_string(),
        ]));
    }
    rows
}
