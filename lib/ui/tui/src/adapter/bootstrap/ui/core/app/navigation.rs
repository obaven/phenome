use crate::bootstrap::panels::dependency_tree::{TreeLine, build_tree_lines};
use crate::bootstrap::state::FocusTarget;

use super::BootstrapApp;

impl BootstrapApp {
    pub(crate) fn move_selection(&mut self, delta: i32) {
        match self.ui.focus {
            FocusTarget::Tree => {
                let registry_specs = self.ports.bootstrap.registry_specs();
                let total_lines = build_tree_lines(
                    self.ports.bootstrap.dependency_graph(),
                    &self.ports.bootstrap.component_states(),
                    &self.ui.collapsed_layers,
                    &registry_specs,
                )
                .len();
                if total_lines == 0 {
                    return;
                }
                let new_index = (self.ui.tree_selected as i32 + delta)
                    .clamp(0, (total_lines - 1) as i32) as usize;
                self.ui.tree_selected = new_index;
            }
            FocusTarget::Status => {
                let total = self.ports.bootstrap.dependency_graph().steps.len();
                if total == 0 {
                    return;
                }
                let new_index =
                    (self.ui.status_selected as i32 + delta).clamp(0, (total - 1) as i32)
                        as usize;
                self.ui.status_selected = new_index;
            }
        }
    }

    pub(crate) fn scroll(&mut self, delta: i32) {
        let scroll = if delta.is_negative() {
            delta.unsigned_abs() as usize
        } else {
            delta as usize
        };

        match self.ui.focus {
            FocusTarget::Tree => {
                if delta.is_negative() {
                    self.ui.tree_scroll = self.ui.tree_scroll.saturating_sub(scroll);
                } else {
                    self.ui.tree_scroll = self.ui.tree_scroll.saturating_add(scroll);
                }
            }
            FocusTarget::Status => {
                if delta.is_negative() {
                    self.ui.status_scroll = self.ui.status_scroll.saturating_sub(scroll);
                } else {
                    self.ui.status_scroll = self.ui.status_scroll.saturating_add(scroll);
                }
            }
        }
    }

    pub(crate) fn toggle_expand_selected(&mut self) {
        if let Some(component) = self.selected_component_id() {
            if self.ui.expanded_components.contains(&component) {
                self.ui.expanded_components.remove(&component);
            } else {
                self.ui.expanded_components.insert(component);
            }
        }
    }

    pub(crate) fn toggle_layer_collapse(&mut self) {
        if self.ui.focus != FocusTarget::Tree {
            return;
        }
        let registry_specs = self.ports.bootstrap.registry_specs();
        let lines = build_tree_lines(
            self.ports.bootstrap.dependency_graph(),
            &self.ports.bootstrap.component_states(),
            &self.ui.collapsed_layers,
            &registry_specs,
        );
        if let Some(TreeLine::Layer { layer, .. }) = lines.get(self.ui.tree_selected) {
            if self.ui.collapsed_layers.contains(layer) {
                self.ui.collapsed_layers.remove(layer);
            } else {
                self.ui.collapsed_layers.insert(*layer);
            }
        }
    }
}
