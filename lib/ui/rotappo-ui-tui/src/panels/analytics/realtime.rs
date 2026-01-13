use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    prelude::Frame,
    style::{Color, Modifier, Style, Stylize},
    widgets::{Block, Borders, Padding, Paragraph},
};

use crate::app::App;
use crate::util::centered_rect;

pub fn render_realtime(frame: &mut Frame, area: Rect, app: &mut App) {
    let app_metrics = app
        .analytics_metrics
        .as_ref()
        .map(|metrics| metrics.as_slice())
        .unwrap_or_default();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Title
            Constraint::Length(10), // Main Stats
            Constraint::Min(0),     // Details
        ])
        .split(area);

    // Title
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

    // Aggregates
    let mut cpu_sum = 0.0;
    let mut mem_sum = 0.0;
    let mut cpu_count = 0;
    let mut mem_count = 0;

    for s in app_metrics {
        match s.metric_type {
            rotappo_domain::MetricType::CpuUsage => {
                cpu_sum += s.value;
                cpu_count += 1;
            }
            rotappo_domain::MetricType::MemoryUsage => {
                mem_sum += s.value;
                mem_count += 1;
            }
            _ => {}
        }
    }

    let cpu_valid = cpu_count > 0;
    let mem_valid = mem_count > 0;

    let stat_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[1]);

    // CPU Card
    let cpu_text = if cpu_valid {
        format!("{:.2} cores", cpu_sum)
    } else {
        "N/A".to_string()
    };
    render_stat_card(
        frame,
        stat_layout[0],
        "Total CPU Load",
        &cpu_text,
        Color::LightGreen,
    );

    // Memory Card
    let mem_text = if mem_valid {
        format_bytes(mem_sum)
    } else {
        "N/A".to_string()
    };
    render_stat_card(
        frame,
        stat_layout[1],
        "Total Memory Usage",
        &mem_text,
        Color::LightMagenta,
    );

    // Footer info
    let info = format!(
        "Samples: {} | Pods: {} | Nodes: {}",
        app_metrics.len(),
        app_metrics
            .iter()
            .filter(|s| matches!(s.resource_type, rotappo_domain::ResourceType::Pod))
            .count()
            / 2, // Approx
        app_metrics
            .iter()
            .filter(|s| matches!(s.resource_type, rotappo_domain::ResourceType::Node))
            .count()
            / 2
    );

    frame.render_widget(
        Paragraph::new(info)
            .style(Style::default().fg(Color::Gray))
            .alignment(Alignment::Center)
            .block(Block::default().padding(Padding::top(1))),
        chunks[2],
    );
}

fn render_stat_card(frame: &mut Frame, area: Rect, title: &str, value: &str, color: Color) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(color))
        .title(title)
        .padding(Padding::uniform(1));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Big Text centered
    let text = Paragraph::new(value)
        .style(Style::default().fg(color).add_modifier(Modifier::BOLD)) // .add_modifier(Modifier::REVERSED) maybe?
        .alignment(Alignment::Center);

    // Center vertically manually-ish or just display at top of padded area
    // For "Big" feel, we might assume standard size or just let Paragraph wrap.
    // Ideally we'd center vertically too, but Ratatui Paragraph doesn't vert-center easily without layout hacks.
    // We'll use a vertical split to push it down.
    let v_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(1),
            Constraint::Length(1),
            Constraint::Min(1),
        ])
        .split(inner);

    frame.render_widget(text, v_chunks[1]);
}

fn format_bytes(bytes: f64) -> String {
    const KI: f64 = 1024.0;
    const MI: f64 = KI * 1024.0;
    const GI: f64 = MI * 1024.0;

    if bytes >= GI {
        format!("{:.2} GiB", bytes / GI)
    } else if bytes >= MI {
        format!("{:.2} MiB", bytes / MI)
    } else if bytes >= KI {
        format!("{:.2} KiB", bytes / KI)
    } else {
        format!("{:.0} B", bytes)
    }
}
