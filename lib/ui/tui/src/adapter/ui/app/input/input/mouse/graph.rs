use crate::app::{App, NavView};

pub(super) fn handle_graph_click(app: &mut App, column: u16, row: u16) -> bool {
    let view = app.active_view();
    if !matches!(view, NavView::TopologyDagGraph | NavView::TopologyDualGraph) {
        return false;
    }
    let area = app.ui.assembly_area;
    if area.width == 0 || area.height == 0 {
        return false;
    }
    if !area.contains((column, row).into()) {
        return false;
    }
    let Some(bounds) = app.graph.view_bounds(app.ui.assembly_area) else {
        return true;
    };
    let width = area.width.saturating_sub(1).max(1);
    let height = area.height.saturating_sub(1).max(1);
    let x_ratio = (column.saturating_sub(area.x) as f64) / (width as f64);
    let y_ratio = (row.saturating_sub(area.y) as f64) / (height as f64);
    let x = bounds.x_min + x_ratio * (bounds.x_max - bounds.x_min);
    let y = bounds.y_max - y_ratio * (bounds.y_max - bounds.y_min);
    if app.graph.select_node_at(x, y) {
        app.ui.show_detail_panel = true;
    }
    true
}
