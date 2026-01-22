use ratatui::text::Line;

use primer::application::timing::compare_runs;
use phenome_ports::PortSet;

pub(super) fn build_comparison_line(ports: &PortSet) -> Option<Line<'static>> {
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
