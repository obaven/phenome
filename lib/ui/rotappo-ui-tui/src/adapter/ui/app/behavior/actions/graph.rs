use phenome_domain::{Event, EventLevel};

use crate::app::App;

impl App {
    pub fn activate_graph_selection(&mut self) {
        let Some(node) = self.graph.selected_node() else {
            return;
        };
        self.runtime.events_mut().push(Event::new(
            EventLevel::Info,
            format!("Topology focus: {}", node.label),
        ));
    }
}
