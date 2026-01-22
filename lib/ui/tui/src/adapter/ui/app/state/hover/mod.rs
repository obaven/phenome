use crate::state::HoverPanel;

use crate::app::App;

mod actions;
mod assembly;
mod capabilities;
mod graph;

impl App {
    pub fn update_hover(&mut self, column: u16, row: u16) {
        let pos = (column, row).into();
        self.ui.mouse_pos = Some((column, row));
        self.ui.hover_panel = HoverPanel::None;
        self.ui.hover_action_index = None;
        self.ui.hover_capability_index = None;
        self.ui.hover_node_id = None;

        if self.ui.assembly_area.contains(pos) && !self.ui.collapsed_assembly_steps {
            let view = self.active_view();
            if matches!(
                view,
                crate::app::NavView::TopologyDagGraph | crate::app::NavView::TopologyDualGraph
            ) {
                graph::update_graph_hover(self, column, row);
            } else {
                self.ui.hover_panel = HoverPanel::Assembly;
                self.ui.hover_action_index = assembly::hover_index_in_assembly(self, row);
            }
        } else if self.ui.capabilities_area.contains(pos) && !self.ui.collapsed_capabilities {
            self.ui.hover_panel = HoverPanel::Capabilities;
            self.ui.hover_capability_index = capabilities::hover_index_in_capabilities(self, row);
        } else if self.ui.actions_area.contains(pos) && !self.ui.collapsed_actions {
            self.ui.hover_panel = HoverPanel::Actions;
            self.ui.hover_action_index = actions::hover_index_in_actions(self, row);
        } else if self.ui.logs_area.contains(pos) && !self.ui.collapsed_logs {
            self.ui.hover_panel = HoverPanel::Logs;
        }
    }
}
