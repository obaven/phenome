//! Assembly rendering helpers.

use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
};

use phenome_domain::{AssemblyStepStatus, CapabilityStatus};
use phenome_ui_presentation::formatting;

use crate::adapter::ui::support::util::spinner_frame;

/// Rendered line for an assembly view.
pub struct AssemblyLine {
    pub line: Line<'static>,
    pub step_index: Option<usize>,
}

/// Icon representing an assembly step status.
///
/// # Examples
/// ```rust
/// use phenome_domain::AssemblyStepStatus;
/// use phenome_ui_tui::util::assembly_status_icon;
///
/// assert_eq!(assembly_status_icon(AssemblyStepStatus::Succeeded), '+');
/// ```
pub fn assembly_status_icon(status: AssemblyStepStatus) -> char {
    match status {
        AssemblyStepStatus::Running => spinner_frame(),
        AssemblyStepStatus::Succeeded => '+',
        AssemblyStepStatus::Failed => 'x',
        AssemblyStepStatus::Blocked => '!',
        AssemblyStepStatus::Pending => '.',
    }
}

/// Icon representing a capability status.
///
/// # Examples
/// ```rust
/// use phenome_domain::CapabilityStatus;
/// use phenome_ui_tui::util::capability_icon;
///
/// assert_eq!(capability_icon(CapabilityStatus::Ready), '+');
/// ```
pub fn capability_icon(status: CapabilityStatus) -> char {
    match status {
        CapabilityStatus::Ready => '+',
        CapabilityStatus::Degraded => '!',
        CapabilityStatus::Offline => 'x',
    }
}

/// Build the formatted assembly lines for display.
pub fn assembly_lines(snapshot: &phenome_domain::Snapshot) -> Vec<AssemblyLine> {
    let mut lines = Vec::new();
    for group in formatting::assembly_groups(snapshot) {
        lines.push(AssemblyLine {
            line: Line::from(Span::styled(
                format!("{domain} domain", domain = group.domain.as_str()),
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )),
            step_index: None,
        });
        for step_info in group.steps {
            let step = &step_info.step;
            let status_style = match step.status {
                AssemblyStepStatus::Succeeded => Style::default().fg(Color::Green),
                AssemblyStepStatus::Running => Style::default().fg(Color::Yellow),
                AssemblyStepStatus::Blocked => Style::default().fg(Color::Red),
                AssemblyStepStatus::Failed => {
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
                }
                AssemblyStepStatus::Pending => Style::default().fg(Color::Gray),
            };
            let pod_text = step
                .pod
                .as_deref()
                .map(|pod| format!(" pod: {pod}"))
                .unwrap_or_else(|| " pod: -".to_string());
            let line = Line::from(vec![
                Span::styled(
                    format!(
                        "[{icon} {status:<9}]",
                        icon = assembly_status_icon(step.status),
                        status = step.status.as_str()
                    ),
                    status_style,
                ),
                Span::raw(" "),
                Span::styled(
                    step.id.clone(),
                    Style::default().add_modifier(Modifier::BOLD),
                ),
                Span::raw(" "),
                Span::raw(step.kind.clone()),
                Span::styled(pod_text, Style::default().fg(Color::DarkGray)),
            ]);
            lines.push(AssemblyLine {
                line,
                step_index: Some(step_info.index),
            });
        }
    }
    lines
}
