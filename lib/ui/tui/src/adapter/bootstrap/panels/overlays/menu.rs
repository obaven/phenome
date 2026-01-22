use crate::bootstrap::state::{BootstrapUiState, MenuAction};
use crate::bootstrap::utils::selected_component_label;
use crate::util::centered_rect;
use ratatui::layout::{Constraint, Rect};
use ratatui::prelude::Frame;
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Row, Table, Wrap};
use phenome_ports::PortSet;

pub fn render(
    frame: &mut Frame,
    area: Rect,
    ports: &PortSet,
    ui: &BootstrapUiState,
    actions: &[MenuAction],
) {
    let overlay_area = centered_rect(70, 70, area);
    frame.render_widget(Clear, overlay_area);

    let block = Block::default()
        .title("Bootstrap Control Menu")
        .borders(Borders::ALL);

    if ui.menu_state.confirming {
        let text = vec![
            Line::from(ui.menu_state.confirmation_message.clone()),
            Line::from(""),
            Line::from("Confirm? [y/N]"),
        ];
        let paragraph = Paragraph::new(text).block(block).wrap(Wrap { trim: true });
        frame.render_widget(paragraph, overlay_area);
        return;
    }

    if let Some(input) = &ui.menu_state.timeout_input {
        let text = vec![
            Line::from("Adjust timeout (seconds):"),
            Line::from(format!("> {input}")),
            Line::from("Press Enter to apply or Esc to cancel"),
        ];
        let paragraph = Paragraph::new(text).block(block).wrap(Wrap { trim: true });
        frame.render_widget(paragraph, overlay_area);
        return;
    }

    let Some(component) = selected_component_label(ports, ui) else {
        return;
    };

    let mut rows = Vec::new();
    rows.push(Row::new(vec![format!("Current component: {component}")]));
    rows.push(Row::new(vec![String::new()]));
    for (idx, action) in actions.iter().enumerate() {
        let prefix = if idx == ui.menu_state.selected {
            ">"
        } else {
            " "
        };
        rows.push(Row::new(vec![format!(
            "{prefix} {label}",
            label = action.label()
        )]));
        rows.push(Row::new(vec![format!(
            "  {desc}",
            desc = action.description()
        )]));
        rows.push(Row::new(vec![String::new()]));
    }

    let table = Table::new(rows, [Constraint::Percentage(100)])
        .block(block)
        .column_spacing(1);

    frame.render_widget(table, overlay_area);
}
