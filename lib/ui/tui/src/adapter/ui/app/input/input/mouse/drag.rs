use crate::app::{App, NavView};

pub(super) fn handle_graph_drag(app: &mut App, column: u16, row: u16) {
    let view = app.active_view();
    if !matches!(view, NavView::TopologyDagGraph | NavView::TopologyDualGraph) {
        return;
    }
    if let Some((prev_c, prev_r)) = app.ui.mouse_pos {
        let dx = (column as i16 - prev_c as i16) as f64;
        let dy = (row as i16 - prev_r as i16) as f64;

        if let Some(layout) = app.graph.layout() {
            let bounds = app.graph.view_bounds_for(layout, app.ui.assembly_area);
            let screen_w = app.ui.assembly_area.width.max(1) as f64;
            let screen_h = app.ui.assembly_area.height.max(1) as f64;

            let graph_dx = dx * (bounds.x_max - bounds.x_min) / screen_w;
            let graph_dy = dy * (bounds.y_max - bounds.y_min) / screen_h;

            app.graph.pan(-graph_dx, graph_dy);
        }
    }
}
