use anyhow::Result;

use super::build::{GraphEdgeRaw, build_layout};
use super::super::types::GraphNode;
use super::tokens::split_plain_tokens;

pub fn parse_plain_layout(text: &str) -> Result<super::GraphLayout> {
    let mut width = 0.0;
    let mut height = 0.0;
    let mut nodes = Vec::new();
    let mut edges = Vec::new();

    for line in text.lines() {
        let tokens = if line.contains('"') {
            split_plain_tokens(line)
        } else {
            line.split_whitespace().map(|s| s.to_string()).collect()
        };
        if tokens.is_empty() {
            continue;
        }
        match tokens[0].as_str() {
            "graph" => {
                if tokens.len() >= 4 {
                    width = tokens[2].parse::<f64>().unwrap_or(width);
                    height = tokens[3].parse::<f64>().unwrap_or(height);
                } else if tokens.len() >= 3 {
                    width = tokens[1].parse::<f64>().unwrap_or(width);
                    height = tokens[2].parse::<f64>().unwrap_or(height);
                }
            }
            "node" => {
                if tokens.len() < 6 {
                    continue;
                }
                let id = tokens[1].clone();
                let x = tokens[2].parse::<f64>().unwrap_or(0.0);
                let y = tokens[3].parse::<f64>().unwrap_or(0.0);
                let node_width = tokens[4].parse::<f64>().unwrap_or(0.5);
                let node_height = tokens[5].parse::<f64>().unwrap_or(0.5);
                let label = tokens.get(6).cloned().unwrap_or_else(|| id.clone());
                nodes.push(GraphNode {
                    id,
                    label,
                    x,
                    y,
                    width: node_width,
                    height: node_height,
                });
            }
            "edge" => {
                if tokens.len() < 5 {
                    continue;
                }
                let tail = tokens[1].clone();
                let head = tokens[2].clone();
                let count = tokens[3].parse::<usize>().unwrap_or(0);
                let mut points = Vec::new();
                let mut idx = 4;
                for _ in 0..count {
                    if idx + 1 >= tokens.len() {
                        break;
                    }
                    let x = tokens[idx].parse::<f64>().unwrap_or(0.0);
                    let y = tokens[idx + 1].parse::<f64>().unwrap_or(0.0);
                    points.push((x, y));
                    idx += 2;
                }
                edges.push(GraphEdgeRaw { tail, head, points });
            }
            "stop" => break,
            _ => {}
        }
    }

    Ok(build_layout(width, height, nodes, edges))
}
