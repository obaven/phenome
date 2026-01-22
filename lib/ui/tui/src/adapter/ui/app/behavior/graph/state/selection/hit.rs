use super::super::core::GraphRenderState;

impl GraphRenderState {
    pub fn node_id_at(&self, x: f64, y: f64) -> Option<String> {
        let layout = self.layout.as_ref()?;

        let exact = layout
            .nodes
            .iter()
            .find(|node| {
                let half_w = node.width / 2.0;
                let half_h = node.height / 2.0;
                x >= node.x - half_w
                    && x <= node.x + half_w
                    && y >= node.y - half_h
                    && y <= node.y + half_h
            })
            .map(|n| n.id.clone());

        if exact.is_some() {
            return exact;
        }

        let radius = 3.0;
        layout
            .nodes
            .iter()
            .filter_map(|node| {
                let dx = x - node.x;
                let dy = y - node.y;
                let dist = (dx * dx + dy * dy).sqrt();
                if dist < radius {
                    Some((dist, node.id.clone()))
                } else {
                    None
                }
            })
            .min_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(_, id)| id)
    }

    pub fn select_node_at(&mut self, x: f64, y: f64) -> bool {
        let Some(layout) = self.layout.as_ref() else {
            return false;
        };

        let exact_match = layout
            .nodes
            .iter()
            .find(|node| {
                let half_w = node.width / 2.0;
                let half_h = node.height / 2.0;
                x >= node.x - half_w
                    && x <= node.x + half_w
                    && y >= node.y - half_h
                    && y <= node.y + half_h
            })
            .map(|n| n.id.clone());

        if let Some(id) = exact_match {
            return self.select_node(&id);
        }

        let radius = 0.5;
        let best_match = layout
            .nodes
            .iter()
            .filter_map(|node| {
                let dx = x - node.x;
                let dy = y - node.y;
                let dist = (dx * dx + dy * dy).sqrt();
                if dist < radius {
                    Some((dist, node.id.clone()))
                } else {
                    None
                }
            })
            .min_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(_, id)| id);

        if let Some(id) = best_match {
            self.select_node(&id)
        } else {
            false
        }
    }
}
