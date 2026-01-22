use std::collections::{HashMap, HashSet};

use super::types::{GraphDependencyPath, GraphEdge, GraphNode};

mod build;
mod parse;
mod tokens;

#[derive(Debug, Clone)]
pub struct GraphLayout {
    pub width: f64,
    pub height: f64,
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
    pub(crate) node_index: HashMap<String, usize>,
    pub(crate) outgoing: Vec<Vec<usize>>,
    pub(crate) incoming: Vec<Vec<usize>>,
}

impl GraphLayout {
    pub fn node(&self, id: &str) -> Option<&GraphNode> {
        let index = self.node_index.get(id)?;
        self.nodes.get(*index)
    }

    pub fn node_index(&self, id: &str) -> Option<usize> {
        self.node_index.get(id).copied()
    }

    pub fn dependency_paths(&self, selected_id: &str) -> GraphDependencyPath {
        let Some(selected_index) = self.node_index(selected_id) else {
            return GraphDependencyPath::default();
        };
        let mut nodes = HashSet::new();
        let mut edges = HashSet::new();
        nodes.insert(selected_index);

        let mut stack = vec![selected_index];
        while let Some(index) = stack.pop() {
            for &edge_index in self.outgoing.get(index).into_iter().flatten() {
                if edges.insert(edge_index) {
                    let head = self.edges[edge_index].head;
                    if nodes.insert(head) {
                        stack.push(head);
                    }
                }
            }
        }

        let mut stack = vec![selected_index];
        while let Some(index) = stack.pop() {
            for &edge_index in self.incoming.get(index).into_iter().flatten() {
                if edges.insert(edge_index) {
                    let tail = self.edges[edge_index].tail;
                    if nodes.insert(tail) {
                        stack.push(tail);
                    }
                }
            }
        }

        GraphDependencyPath { nodes, edges }
    }
}

pub use parse::parse_plain_layout;
