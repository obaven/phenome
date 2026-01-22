use ratatui::layout::Rect;
use ratatui::prelude::Frame;

use crate::app::App;
use primer::application::flows::reconcile::visualize;

mod detail;
mod draw;
mod layout;
mod overlay;

pub(super) fn render_topology_graph(
    frame: &mut Frame,
    area: Rect,
    app: &mut App,
    view: visualize::ViewType,
    label: &str,
) {
    let (graph_area, sidebar_area, dot) = layout::prepare_graph(app, area, view);

    if let Some(layout) = app.graph.layout() {
        draw::render_canvas(frame, graph_area, app, layout);
        if let Some(sidebar) = sidebar_area {
            detail::render_detail_sidebar(frame, sidebar, app);
        }
        if app.ui.search_active {
            overlay::render_search_overlay(frame, graph_area, app);
        }
        return;
    }

    layout::render_dot_fallback(frame, area, app, label, &dot);
}
