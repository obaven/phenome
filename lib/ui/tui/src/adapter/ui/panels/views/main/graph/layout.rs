use std::collections::HashMap;

use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::prelude::Frame;
use ratatui::text::Line;
use ratatui::widgets::{Paragraph, Wrap};

use crate::app::App;
use crate::panels::views::main::shared::section_title;
use primer::application::flows::reconcile::visualize;

pub(super) fn prepare_graph(
    app: &mut App,
    area: Rect,
    view: visualize::ViewType,
) -> (Rect, Option<Rect>, String) {
    let (graph_area, sidebar_area) = if app.ui.show_detail_panel {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(10),
                Constraint::Length(15),
            ])
            .split(area);
        app.ui.detail_area = chunks[1];
        (chunks[0], Some(chunks[1]))
    } else {
        app.ui.detail_area = Rect::default();
        (area, None)
    };

    app.ui.assembly_area = graph_area;
    app.ui.collapsed_assembly_steps = false;
    let assembly = app.context.ports.bootstrap.dependency_graph();
    let (graph, node_map) = visualize::graph::build_filtered_graph(assembly, view);

    let index_map: HashMap<_, _> = node_map.iter().map(|(k, v)| (*v, k.clone())).collect();
    let dot = visualize::render::generate_pretty_dot(&graph, &index_map);

    if let Err(error) = app.graph.ensure_layout(&dot) {
        app.graph.mark_layout_failed(error.to_string());
    }

    app.graph.queue_request(graph_area, dot.clone());
    (graph_area, sidebar_area, dot)
}

pub(super) fn render_dot_fallback(
    frame: &mut Frame,
    area: Rect,
    app: &App,
    label: &str,
    dot: &str,
) {
    let mut lines = Vec::new();
    lines.push(section_title(label));
    if let Some(error) = app.graph.layout_error() {
        lines.push(Line::from(format!("Interactive layout failed: {error}")));
        lines.push(Line::from(""));
    }
    for line in dot.lines() {
        lines.push(Line::from(line.to_string()));
    }
    let paragraph = Paragraph::new(lines)
        .wrap(Wrap { trim: false })
        .scroll((app.ui.assembly_scroll, 0));
    frame.render_widget(paragraph, area);
}
