use anyhow::{Context, Result};

use super::super::core::GraphRenderState;
use super::super::super::layout::{GraphLayout, parse_plain_layout};
use super::super::super::render::{hash_dot, render_dot_plain};

impl GraphRenderState {
    pub fn ensure_layout(&mut self, dot: &str) -> Result<()> {
        let hash = hash_dot(dot);
        if self.layout_hash == Some(hash) {
            return Ok(());
        }
        let plain = render_dot_plain(dot).context("graphviz plain render failed")?;
        let layout = parse_plain_layout(&plain).context("graphviz plain parse failed")?;
        let previous = self.selected_id.clone();
        self.selected_id = previous
            .filter(|id| layout.node_index.contains_key(id))
            .or_else(|| layout.nodes.first().map(|node| node.id.clone()));
        self.layout = Some(layout);
        self.layout_hash = Some(hash);
        self.layout_error = None;
        Ok(())
    }

    pub fn layout(&self) -> Option<&GraphLayout> {
        self.layout.as_ref()
    }

    pub fn layout_error(&self) -> Option<&str> {
        self.layout_error.as_deref()
    }

    pub fn mark_layout_failed(&mut self, error: String) {
        self.layout_error = Some(error);
        self.layout = None;
        self.layout_hash = None;
    }
}
