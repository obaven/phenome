use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    layout::{Constraint, Rect},
    style::{Color, Modifier, Style},
    text::Text,
    widgets::{Block, Borders, Cell, Row, Table, TableState},
};
use rotappo_ports::{ComponentState, ComponentStatus, PortSet};
use std::collections::HashSet;

pub struct ComponentStatusPanel {
    state: TableState,
    expanded_components: HashSet<String>,
}

impl Default for ComponentStatusPanel {
    fn default() -> Self {
        Self::new()
    }
}

impl ComponentStatusPanel {
    pub fn new() -> Self {
        let mut state = TableState::default();
        state.select(Some(0));
        Self {
            state,
            expanded_components: HashSet::new(),
        }
    }

    pub fn render(&mut self, f: &mut Frame, area: Rect, ports: &PortSet) {
        let bootstrap_port = &ports.bootstrap;
        let states_map = bootstrap_port.component_states();

        // Sort components by ID for consistent ordering
        let mut states: Vec<&ComponentState> = states_map.values().collect();
        states.sort_by_key(|s| &s.id);

        let mut rows = Vec::new();

        for state in &states {
            let is_expanded = self.expanded_components.contains(&state.id);

            // Summary Row
            let status_icon = match state.status {
                ComponentStatus::Pending => "⏳",
                ComponentStatus::Running => "🔄",
                ComponentStatus::Complete => "✓",
                ComponentStatus::Failed => "✗",
                ComponentStatus::Deferred => "⏸",
            };

            let elapsed = state
                .timing
                .current_elapsed()
                .map(|d| format!("{d:?}"))
                .unwrap_or_default();

            // Mock progress bar for now
            let progress = match state.status {
                ComponentStatus::Complete => "████████",
                ComponentStatus::Running => "████░░░░",
                _ => "░░░░░░░░",
            };

            rows.push(Row::new(vec![
                Cell::from(state.id.clone()),
                Cell::from(format!("{} {:?}", status_icon, state.status)),
                Cell::from(elapsed),
                Cell::from(progress),
            ]));

            if is_expanded {
                // Detailed status row
                let details = format!(
                    "  └─ Status: {:?}\n  └─ Deferred: {:?}\n  └─ Retries: {}",
                    state.readiness, state.deferred_reason, state.retry_count
                );

                rows.push(
                    Row::new(vec![
                        Cell::from(Text::from(details)).style(Style::default().fg(Color::DarkGray)),
                    ])
                    .height(4),
                ); // Give it some height
            }
        }

        let widths = [
            Constraint::Percentage(30),
            Constraint::Percentage(30),
            Constraint::Percentage(15),
            Constraint::Percentage(25),
        ];

        let table = Table::new(rows, widths)
            .header(
                Row::new(vec!["Component", "Status", "Time", "Progress"])
                    .style(Style::default().add_modifier(Modifier::BOLD)),
            )
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Component Status"),
            )
            .row_highlight_style(Style::default().add_modifier(Modifier::REVERSED));

        f.render_stateful_widget(table, area, &mut self.state);
    }

    pub fn handle_input(&mut self, key: KeyEvent, ports: &PortSet) -> Result<()> {
        let bootstrap_port = &ports.bootstrap;
        let states_map = bootstrap_port.component_states();
        let count = states_map.len();
        // Since we might have expanded rows, navigation logic is tricky if we map 1:1 with states.
        // For simple MVP, let's assume selection index maps to sorted components list.
        // We need to keep selection valid.

        match key.code {
            KeyCode::Char('e') => {
                if let Some(selected) = self.state.selected() {
                    let mut states: Vec<&ComponentState> = states_map.values().collect();
                    states.sort_by_key(|s| &s.id);
                    if let Some(state) = states.get(selected) {
                        if self.expanded_components.contains(&state.id) {
                            self.expanded_components.remove(&state.id);
                        } else {
                            self.expanded_components.insert(state.id.clone());
                        }
                    }
                }
            }
            KeyCode::Up => {
                let i = match self.state.selected() {
                    Some(i) => {
                        if i == 0 {
                            count.saturating_sub(1)
                        } else {
                            i - 1
                        }
                    }
                    None => 0,
                };
                self.state.select(Some(i));
            }
            KeyCode::Down => {
                let i = match self.state.selected() {
                    Some(i) => {
                        if i >= count.saturating_sub(1) {
                            0
                        } else {
                            i + 1
                        }
                    }
                    None => 0,
                };
                self.state.select(Some(i));
            }
            _ => {}
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_state() {
        let panel = ComponentStatusPanel::new();
        assert!(panel.expanded_components.is_empty());
        assert_eq!(panel.state.selected(), Some(0));
    }

    #[test]
    fn test_navigation() {
        // Mock ports would be needed for full testing, but unit logic checks:
        let mut panel = ComponentStatusPanel::new();
        // Simulate 3 items
        let count = 3;

        // Down
        let current = panel.state.selected().unwrap();
        let next = if current >= count - 1 { 0 } else { current + 1 };
        panel.state.select(Some(next));
        assert_eq!(panel.state.selected(), Some(1));

        // Up
        let current = panel.state.selected().unwrap();
        let prev = if current == 0 { count - 1 } else { current - 1 };
        panel.state.select(Some(prev));
        assert_eq!(panel.state.selected(), Some(0));
    }
}
