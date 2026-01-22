use super::super::core::GraphRenderState;
use super::super::super::types::GraphNode;

impl GraphRenderState {
    pub fn selected_id(&self) -> Option<&str> {
        self.selected_id.as_deref()
    }

    pub fn selected_node(&self) -> Option<&GraphNode> {
        let id = self.selected_id.as_deref()?;
        self.layout.as_ref()?.node(id)
    }

    pub fn select_node(&mut self, id: &str) -> bool {
        if self.selected_id.as_deref() == Some(id) {
            return false;
        }
        self.selected_id = Some(id.to_string());

        if let Some(layout) = self.layout.as_ref() {
            if let Some(node) = layout.node(id) {
                let graph_center_x = layout.width / 2.0;
                let graph_center_y = layout.height / 2.0;
                self.pan_x = node.x - graph_center_x;
                self.pan_y = node.y - graph_center_y;
            }
        }
        true
    }
}
