use ratatui::layout::Rect;
use ratatui::prelude::Frame;
use ratatui::style::Color;
use ratatui::symbols::Marker;
use ratatui::widgets::canvas::{Canvas, Line, Rectangle};

use crate::app::graph::GraphLayout;
use crate::app::App;

pub(super) fn render_canvas(frame: &mut Frame, area: Rect, app: &App, layout: &GraphLayout) {
    let bounds = app.graph.view_bounds_for(layout, area);
    let selected = app.graph.selected_id();
    let dependency = selected
        .map(|id| layout.dependency_paths(id))
        .unwrap_or_default();
    let selected_id = selected.map(|id| id.to_string());
    let image_active = app.graph.image_active();

    let canvas = Canvas::default()
        .marker(Marker::Braille)
        .x_bounds([bounds.x_min, bounds.x_max])
        .y_bounds([bounds.y_min, bounds.y_max])
        .paint(move |ctx| {
            if !image_active {
                for (i, edge) in layout.edges.iter().enumerate() {
                    let color = if dependency.edges.contains(&i) {
                        Color::Cyan
                    } else {
                        Color::Gray
                    };
                    for i in 0..edge.points.len().saturating_sub(1) {
                        let p1 = edge.points[i];
                        let p2 = edge.points[i + 1];
                        ctx.draw(&Line {
                            x1: p1.0,
                            y1: p1.1,
                            x2: p2.0,
                            y2: p2.1,
                            color,
                        });
                    }
                }

                for (i, node) in layout.nodes.iter().enumerate() {
                    let color = if selected_id.as_deref() == Some(node.id.as_str()) {
                        Color::Yellow
                    } else if dependency.nodes.contains(&i) {
                        Color::Cyan
                    } else {
                        Color::Blue
                    };

                    let rect = Rectangle {
                        x: node.x - node.width / 2.0,
                        y: node.y - node.height / 2.0,
                        width: node.width,
                        height: node.height,
                        color,
                    };
                    ctx.draw(&rect);
                }
            } else if let Some(selected_id) = selected {
                if let Some(node) = layout.node(selected_id) {
                    let rect = Rectangle {
                        x: node.x - node.width / 2.0,
                        y: node.y - node.height / 2.0,
                        width: node.width,
                        height: node.height,
                        color: Color::Yellow,
                    };
                    ctx.draw(&rect);
                }
            }
        });

    frame.render_widget(canvas, area);
}
