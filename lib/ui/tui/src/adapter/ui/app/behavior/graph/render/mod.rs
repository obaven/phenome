use anyhow::{Context, Result};
use graphviz_rust::cmd::{CommandArg, Format, Layout};
use graphviz_rust::printer::PrinterContext;
use graphviz_rust::{exec, parse};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

pub(super) fn hash_dot(dot: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    dot.hash(&mut hasher);
    hasher.finish()
}

pub(super) fn render_dot_with_args(dot: &str, args: Vec<CommandArg>) -> Result<Vec<u8>> {
    let graph = parse(dot).map_err(|e| anyhow::anyhow!("failed to parse DOT: {e}"))?;
    let bytes =
        exec(graph, &mut PrinterContext::default(), args).context("failed to execute graphviz")?;
    Ok(bytes)
}

pub(super) fn render_dot_plain(dot: &str) -> Result<String> {
    let graph = parse(dot).map_err(|e| anyhow::anyhow!("failed to parse DOT: {e}"))?;
    let bytes = exec(
        graph,
        &mut PrinterContext::default(),
        vec![
            CommandArg::Format(Format::Plain),
            CommandArg::Layout(Layout::Dot),
        ],
    )
    .context("failed to execute graphviz")?;
    let text = String::from_utf8(bytes).context("plain output is not utf-8")?;
    Ok(text)
}

#[cfg(test)]
mod tests {
    use super::render_dot_plain;
    use crate::app::{GraphRenderState, TerminalImageProtocol, graph::GraphRenderStatus};
    use ratatui::layout::Rect;

    #[test]
    fn test_graphviz_installed() {
        let dot = "digraph G { a -> b; }";
        let plain = render_dot_plain(dot);
        assert!(
            plain.is_ok(),
            "Graphviz 'dot' command failed. Is graphviz installed? Error: {:?}",
            plain.err()
        );
    }

    #[test]
    fn test_ensure_layout() {
        let mut state = GraphRenderState::new();
        let dot = "digraph G { a -> b; }";
        let res = state.ensure_layout(dot);
        assert!(res.is_ok(), "ensure_layout failed: {:?}", res.err());
        assert!(state.layout().is_some(), "Layout should be populated");
        let layout = state.layout().unwrap();
        assert_eq!(layout.nodes.len(), 2, "Should have 2 nodes");
    }

    #[test]
    fn test_ensure_image_generation() {
        let mut state = GraphRenderState::new();
        state.protocol = TerminalImageProtocol::Kitty;

        let dot = "digraph G { a -> b; }";
        state.queue_request(Rect::new(0, 0, 100, 100), dot.to_string());

        let res = state.ensure_image();
        assert!(res.is_ok(), "ensure_image failed: {:?}", res.err());

        assert!(state.image().is_some(), "Image bytes should be present");
        assert!(
            state.image().unwrap().len() > 0,
            "Image should not be empty"
        );
        assert_eq!(state.status(), GraphRenderStatus::Rendered);
    }
}
