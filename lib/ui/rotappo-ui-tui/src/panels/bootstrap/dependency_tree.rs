use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    layout::Rect,
    widgets::{Block, Borders, List, ListItem, ListState},
};
use rotappo_ports::PortSet;
use std::collections::HashSet;

pub struct DependencyTreePanel {
    collapsed_layers: HashSet<String>,
    state: ListState,
    items: Vec<TreeItem>,
}

#[derive(Clone, Debug)]
enum TreeItem {
    Layer {
        name: String,
        completed: usize,
        total: usize,
    },
    Component {
        id: String,
        status: String,
        elapsed: String,
        _layer: String,
    },
}

impl Default for DependencyTreePanel {
    fn default() -> Self {
        Self::new()
    }
}

impl DependencyTreePanel {
    pub fn new() -> Self {
        Self {
            collapsed_layers: HashSet::new(),
            state: ListState::default(),
            items: Vec::new(),
        }
    }

    pub fn render(&mut self, f: &mut Frame, area: Rect, ports: &PortSet) {
        let bootstrap_port = &ports.bootstrap;
        let assembly = bootstrap_port.dependency_graph();
        let states = bootstrap_port.component_states();

        // Regenerate items based on current state and collapsed layers
        self.items = self.build_tree_items(assembly, &states);

        // Ensure selection is valid
        if self.state.selected().is_none() && !self.items.is_empty() {
            self.state.select(Some(0));
        }

        let list_items: Vec<ListItem> = self
            .items
            .iter()
            .map(|item| match item {
                TreeItem::Layer {
                    name,
                    completed,
                    total,
                } => {
                    let icon = if *completed == *total { "✓" } else { " " };
                    let collapsed = if self.collapsed_layers.contains(name) {
                        "+"
                    } else {
                        "-"
                    };
                    ListItem::new(format!("{collapsed} {name} [{completed}/{total}] {icon}"))
                }
                TreeItem::Component {
                    id,
                    status,
                    elapsed,
                    ..
                } => ListItem::new(format!("  {status} {id} ({elapsed})")),
            })
            .collect();

        let list = List::new(list_items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Dependency Tree"),
            )
            .highlight_symbol("> ");

        f.render_stateful_widget(list, area, &mut self.state);
    }

    pub fn handle_input(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Char('c') => self.toggle_collapse(),
            KeyCode::Up => self.previous(),
            KeyCode::Down => self.next(),
            _ => {}
        }
        Ok(())
    }

    fn build_tree_items(
        &self,
        assembly: &bootstrappo::domain::models::assembly::Assembly,
        states: &std::collections::HashMap<String, rotappo_ports::ComponentState>,
    ) -> Vec<TreeItem> {
        let mut items = Vec::new();
        // TODO: Implement proper layering logic from assembly metadata
        // For now, grouping everything under "Default" layer for MVP structure
        let layer_name = "Default";

        let components: Vec<&bootstrappo::domain::models::assembly::Step> =
            assembly.steps.iter().collect();
        let total = components.len();
        let completed = components
            .iter()
            .filter(|s| {
                states
                    .get(&s.id)
                    .map(|st| st.status == rotappo_ports::ComponentStatus::Complete)
                    .unwrap_or(false)
            })
            .count();

        items.push(TreeItem::Layer {
            name: layer_name.to_string(),
            completed,
            total,
        });

        if !self.collapsed_layers.contains(layer_name) {
            for step in components {
                let status = states
                    .get(&step.id)
                    .map(|s| format!("{:?}", s.status))
                    .unwrap_or("Pending".to_string());
                let elapsed = states
                    .get(&step.id)
                    .and_then(|s| s.timing.current_elapsed())
                    .map(|d| format!("{d:?}"))
                    .unwrap_or("".to_string());

                items.push(TreeItem::Component {
                    id: step.id.clone(),
                    status,
                    elapsed,
                    _layer: layer_name.to_string(),
                });
            }
        }
        items
    }

    fn toggle_collapse(&mut self) {
        if let Some(idx) = self.state.selected() {
            if let Some(TreeItem::Layer { name, .. }) = self.items.get(idx) {
                if self.collapsed_layers.contains(name) {
                    self.collapsed_layers.remove(name);
                } else {
                    self.collapsed_layers.insert(name.clone());
                }
            }
        }
    }

    fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bootstrappo::domain::models::assembly::{Assembly, Step};
    use std::collections::HashMap;

    fn create_test_step(id: &str) -> Step {
        Step {
            id: id.to_string(),
            kind: "test".to_string(),
            required: vec![],
            provides: vec![],
            checks: vec![],
            helm: None,
            kro: None,
            terraform: None,
            lock_scope: None,
        }
    }

    #[test]
    fn test_initial_state() {
        let panel = DependencyTreePanel::new();
        assert!(panel.collapsed_layers.is_empty());
        assert!(panel.items.is_empty());
    }

    #[test]
    fn test_build_tree_items() {
        let mut assembly = Assembly::default();
        assembly.steps.push(create_test_step("step1"));

        let states = HashMap::new();
        let panel = DependencyTreePanel::new();

        let items = panel.build_tree_items(&assembly, &states);

        // Should have 1 layer + 1 component
        assert_eq!(items.len(), 2);

        match &items[0] {
            TreeItem::Layer { name, total, .. } => {
                assert_eq!(name, "Default");
                assert_eq!(*total, 1);
            }
            _ => panic!("Expected Layer"),
        }

        match &items[1] {
            TreeItem::Component { id, .. } => {
                assert_eq!(id, "step1");
            }
            _ => panic!("Expected Component"),
        }
    }

    #[test]
    fn test_collapse_layer() {
        let mut panel = DependencyTreePanel::new();
        let mut assembly = Assembly::default();
        assembly.steps.push(create_test_step("step1"));
        let states = HashMap::new();

        // Initial build
        panel.items = panel.build_tree_items(&assembly, &states);
        panel.state.select(Some(0)); // Select layer

        // Toggle collapse
        panel.toggle_collapse();
        assert!(panel.collapsed_layers.contains("Default"));

        // Rebuild
        panel.items = panel.build_tree_items(&assembly, &states);
        assert_eq!(panel.items.len(), 1); // Only layer
    }
}
