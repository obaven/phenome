use ratatui::style::{Color, Style, Stylize};
use ratatui::text::{Line, Span};
use phenome_ports::{ComponentStatus, PortSet};

use crate::bootstrap::utils::format_duration;

use super::comparison;

pub(super) fn build_overall_text(ports: &PortSet) -> Vec<Line<'static>> {
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
    if let Some(line) = comparison::build_comparison_line(ports) {
        overall_text.push(Line::from(""));
        overall_text.push(line);
    }
    overall_text
}
