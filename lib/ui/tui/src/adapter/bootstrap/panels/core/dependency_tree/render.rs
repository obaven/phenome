use ratatui::layout::Rect;
use ratatui::prelude::Frame;
use ratatui::style::{Color, Style};
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use phenome_ports::{ComponentStatus, PortSet};

use crate::bootstrap::state::{BootstrapUiState, FocusTarget};
use crate::bootstrap::utils::{format_duration, layer_label, status_icon};

use super::TreeLine;
use super::tree::build_tree_lines;

pub fn render(frame: &mut Frame, area: Rect, ports: &PortSet, ui: &BootstrapUiState) {
    let states = ports.bootstrap.component_states();
    let registry_specs = ports.bootstrap.registry_specs();
    let lines = build_tree_lines(
        ports.bootstrap.dependency_graph(),
        &states,
        &ui.collapsed_layers,
        &registry_specs,
    );
    let start = ui.tree_scroll.min(lines.len().saturating_sub(1));
    let end = (start + area.height as usize).min(lines.len());

    let mut rendered = Vec::new();
    for (idx, line) in lines[start..end].iter().cloned().enumerate() {
        let line_index = start + idx;
        rendered.push(format_tree_line(
            line,
            ui.focus == FocusTarget::Tree && line_index == ui.tree_selected,
        ));
    }

    let block = Block::default()
        .title("Dependency Tree")
        .borders(Borders::ALL);
    let paragraph = Paragraph::new(rendered)
        .block(block)
        .wrap(Wrap { trim: true });
    frame.render_widget(paragraph, area);
}

fn format_tree_line(line: TreeLine, selected: bool) -> Line<'static> {
    let (text, style) = match line {
        TreeLine::Layer {
            layer,
            total,
            completed,
        } => {
            let status_tag = if completed == total { "OK" } else { "PEND" };
            let label = format!(
                "{} [{} {}/{}]",
                layer_label(layer),
                status_tag,
                completed,
                total
            );
            (label, Style::default().fg(Color::Yellow))
        }
        TreeLine::Component {
            id,
            status,
            elapsed,
        } => {
            let icon = status_icon(status);
            let elapsed_text = elapsed
                .map(format_duration)
                .unwrap_or_else(|| "-".to_string());
            let label = format!("  {icon} {id} ({elapsed_text})");
            let style = match status {
                ComponentStatus::Running => Style::default().fg(Color::Cyan),
                ComponentStatus::Failed => Style::default().fg(Color::Red),
                ComponentStatus::Deferred => Style::default().fg(Color::DarkGray),
                ComponentStatus::Complete => Style::default().fg(Color::Green),
                ComponentStatus::Pending => Style::default().fg(Color::White),
            };
            (label, style)
        }
    };

    let styled = if selected {
        Style::default().bg(Color::Blue).fg(Color::White)
    } else {
        style
    };
    Line::styled(text, styled)
}
