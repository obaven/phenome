use crate::bootstrap::state::{BootstrapUiState, FocusTarget};
use crate::bootstrap::utils::{
    format_duration, format_row, format_status, progress_bar, slice_lines, style_line, table_widths,
};
use ratatui::layout::Rect;
use ratatui::prelude::Frame;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use phenome_ports::{ComponentState, PortSet};

pub fn render(frame: &mut Frame, area: Rect, ports: &PortSet, ui: &mut BootstrapUiState) {
    let assembly = ports.bootstrap.dependency_graph();
    let states = ports.bootstrap.component_states();
    let mut lines = Vec::new();

    let widths = table_widths(area.width);
    let header = format_row(&["Component", "Status", "Time", "Progress"], &widths);
    lines.push(Line::styled(
        header,
        Style::default()
            .fg(Color::DarkGray)
            .add_modifier(Modifier::BOLD),
    ));

    for (index, step) in assembly.steps.iter().enumerate() {
        let state = states
            .get(&step.id)
            .cloned()
            .unwrap_or_else(|| ComponentState::new(step.id.clone()));
        let summary = format_component_summary(&state, &widths);
        let selected = ui.focus == FocusTarget::Status && index == ui.status_selected;
        lines.push(style_line(summary, selected));

        if ui.expanded_components.contains(&step.id) {
            if let Ok(details) = ports.bootstrap.get_detailed_status(&step.id) {
                let detail_lines = format_component_details(&details, &widths);
                for detail in detail_lines {
                    lines.push(Line::from(detail));
                }
            }
        }
    }

    let visible_lines = slice_lines(&lines, ui.status_scroll, area.height as usize);
    let paragraph = Paragraph::new(visible_lines)
        .block(
            Block::default()
                .title("Component Status")
                .borders(Borders::ALL),
        )
        .wrap(Wrap { trim: true });
    frame.render_widget(paragraph, area);
}

fn format_component_summary(state: &ComponentState, widths: &[usize; 4]) -> String {
    let status_text = format_status(state);
    let elapsed_text = state
        .timing
        .current_elapsed()
        .map(format_duration)
        .unwrap_or_else(|| "-".to_string());
    let progress = state
        .readiness
        .as_ref()
        .and_then(|status| status.basic.progress)
        .unwrap_or(0.0);
    let progress_text = progress_bar(progress, 8);

    format_row(
        &[&state.id, &status_text, &elapsed_text, &progress_text],
        widths,
    )
}

fn format_component_details(
    details: &primer::application::readiness::DetailedStatus,
    widths: &[usize; 4],
) -> Vec<String> {
    let mut lines = Vec::new();
    for pod in details.pods.iter().take(4) {
        let ready = if pod.ready { "Ready" } else { "NotReady" };
        lines.push(format!(
            "  - Pod {name}: {phase} ({ready})",
            name = pod.name.as_str(),
            phase = pod.phase.as_str()
        ));
    }

    if lines.is_empty() {
        lines.push(format_row(
            &["  - No detailed status available", "", "", ""],
            widths,
        ));
    }
    lines
}
