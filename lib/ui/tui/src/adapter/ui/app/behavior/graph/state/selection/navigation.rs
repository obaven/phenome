use super::super::core::GraphRenderState;
use super::super::super::types::GraphDirection;

impl GraphRenderState {
    pub fn select_next(&mut self) -> bool {
        let next_id = {
            let Some(layout) = self.layout.as_ref() else {
                return false;
            };
            if layout.nodes.is_empty() {
                return false;
            }
            let next_index = match self.selected_id.as_deref() {
                Some(id) => layout
                    .node_index(id)
                    .map(|index| (index + 1) % layout.nodes.len())
                    .unwrap_or(0),
                None => 0,
            };
            layout.nodes[next_index].id.clone()
        };
        self.select_node(&next_id)
    }

    pub fn select_prev(&mut self) -> bool {
        let prev_id = {
            let Some(layout) = self.layout.as_ref() else {
                return false;
            };
            if layout.nodes.is_empty() {
                return false;
            }
            let prev_index = match self.selected_id.as_deref() {
                Some(id) => layout
                    .node_index(id)
                    .map(|index| (index + layout.nodes.len() - 1) % layout.nodes.len())
                    .unwrap_or(0),
                None => 0,
            };
            layout.nodes[prev_index].id.clone()
        };
        self.select_node(&prev_id)
    }

    pub fn select_direction(&mut self, direction: GraphDirection) -> bool {
        let best_id = {
            let Some(layout) = self.layout.as_ref() else {
                return false;
            };
            let Some(current_id) = self.selected_id.as_deref() else {
                return self.select_next();
            };
            let Some(current) = layout.node(current_id) else {
                return false;
            };
            let dir = match direction {
                GraphDirection::Left => (-1.0, 0.0),
                GraphDirection::Right => (1.0, 0.0),
                GraphDirection::Up => (0.0, 1.0),
                GraphDirection::Down => (0.0, -1.0),
            };
            let mut best = None;
            for node in &layout.nodes {
                if node.id == current.id {
                    continue;
                }
                let dx = node.x - current.x;
                let dy = node.y - current.y;
                let dist = (dx * dx + dy * dy).sqrt();
                if dist == 0.0 {
                    continue;
                }
                let alignment = (dx * dir.0 + dy * dir.1) / dist;
                if alignment < 0.3 {
                    continue;
                }
                let score = (1.0 - alignment) * 10.0 + dist;
                if best
                    .as_ref()
                    .map(|(_, best_score)| score < *best_score)
                    .unwrap_or(true)
                {
                    best = Some((node.id.clone(), score));
                }
            }
            best.map(|(id, _)| id)
        };

        if let Some(id) = best_id {
            self.select_node(&id)
        } else {
            false
        }
    }
}
