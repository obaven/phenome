use crate::state::HoverPanel;

use crate::app::App;

pub(super) fn update_graph_hover(app: &mut App, column: u16, row: u16) {
    if let Some(bounds) = app.graph.view_bounds(app.ui.assembly_area) {
        let area = app.ui.assembly_area;
        let width = area.width.max(1) as f64;
        let height = area.height.max(1) as f64;
        let x_ratio = (column.saturating_sub(area.x) as f64 + 0.5) / width;
        let y_ratio = (row.saturating_sub(area.y) as f64 + 0.5) / height;
        let x = bounds.x_min + x_ratio * (bounds.x_max - bounds.x_min);
        let y = bounds.y_max - y_ratio * (bounds.y_max - bounds.y_min);

        app.ui.hover_node_id = app.graph.node_id_at(x, y);
        if app.ui.hover_node_id.is_some() {
            app.ui.hover_panel = HoverPanel::Graph;
        }
    }
}
