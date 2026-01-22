use std::collections::HashMap;

use super::super::types::{GraphEdge, GraphNode};
use super::GraphLayout;

pub(super) struct GraphEdgeRaw {
    pub(super) tail: String,
    pub(super) head: String,
    pub(super) points: Vec<(f64, f64)>,
}

pub(super) fn build_layout(
    width: f64,
    height: f64,
    nodes: Vec<GraphNode>,
    edges: Vec<GraphEdgeRaw>,
) -> GraphLayout {
    let mut node_index = HashMap::new();
    for (idx, node) in nodes.iter().enumerate() {
        node_index.insert(node.id.clone(), idx);
    }

    let mut outgoing = vec![Vec::new(); nodes.len()];
    let mut incoming = vec![Vec::new(); nodes.len()];
    let mut resolved_edges = Vec::new();
    for edge in edges {
        let Some(&tail) = node_index.get(&edge.tail) else {
            continue;
        };
        let Some(&head) = node_index.get(&edge.head) else {
            continue;
        };
        let edge_index = resolved_edges.len();
        resolved_edges.push(GraphEdge {
            tail,
            head,
            points: edge.points,
        });
        outgoing[tail].push(edge_index);
        incoming[head].push(edge_index);
    }

    GraphLayout {
        width,
        height,
        nodes,
        edges: resolved_edges,
        node_index,
        outgoing,
        incoming,
    }
}
