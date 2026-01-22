use crate::bootstrap::utils::{format_duration, progress_bar};
use ratatui::layout::Rect;
use ratatui::prelude::Frame;
use ratatui::style::{Color, Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use phenome_ports::{ComponentStatus, PortSet};

pub fn render(frame: &mut Frame, area: Rect, ports: &PortSet) {
    let status = ports.bootstrap.bootstrap_status();
    let states = ports.bootstrap.component_states();
    let total = status
        .total_components
        .unwrap_or_else(|| states.len().max(1));
    let completed = states
        .values()
        .filter(|s| s.status == ComponentStatus::Complete)
        .count();
    let running = states
        .values()
        .filter(|s| s.status == ComponentStatus::Running)
        .count();
    let pending = states
        .values()
        .filter(|s| s.status == ComponentStatus::Pending)
        .count();
    let failed = states
        .values()
        .filter(|s| s.status == ComponentStatus::Failed)
        .count();
    let deferred = states
        .values()
        .filter(|s| s.status == ComponentStatus::Deferred)
        .count();

    let elapsed = status
        .started_at
        .map(|start| start.elapsed())
        .unwrap_or_default();
    let elapsed_text = format_duration(elapsed);

    let progress = if total == 0 {
        0.0
    } else {
        completed as f32 / total as f32
    };

    let lines = vec![
        Line::from(vec![
            Span::styled(
                "Bootstrappo - Bootstrap",
                Style::default().fg(Color::Cyan).bold(),
            ),
            Span::raw("  "),
            Span::raw(format!("Elapsed: {elapsed_text}")),
            Span::raw("  "),
            Span::raw(format!("OK {completed}/{total}")),
            Span::raw("  "),
            Span::raw(format!("RUN {running}")),
            Span::raw("  "),
            Span::raw(format!("PEND {pending}")),
            Span::raw("  "),
            Span::raw(format!("DEF {deferred}")),
            Span::raw("  "),
            Span::raw(format!("FAIL {failed}")),
        ]),
        Line::from(progress_bar(
            progress,
            area.width.saturating_sub(4) as usize,
        )),
    ];

    let block = Block::default().borders(Borders::ALL);
    let paragraph = Paragraph::new(lines).block(block);
    frame.render_widget(paragraph, area);
}
