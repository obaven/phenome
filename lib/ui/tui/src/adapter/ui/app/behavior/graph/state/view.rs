use ratatui::layout::Rect;

use super::core::GraphRenderState;
use super::super::layout::GraphLayout;
use super::super::types::GraphBounds;

impl GraphRenderState {
    pub fn zoom_in(&mut self) {
        self.zoom = (self.zoom * 1.2).min(4.0);
    }

    pub fn zoom_out(&mut self) {
        self.zoom = (self.zoom / 1.2).max(0.4);
    }

    pub fn reset_view(&mut self) {
        self.zoom = 1.0;
        self.pan_x = 0.0;
        self.pan_y = 0.0;
    }

    pub fn pan(&mut self, dx: f64, dy: f64) {
        self.pan_x += dx;
        self.pan_y += dy;
    }

    pub fn view_bounds(&self, area: Rect) -> Option<GraphBounds> {
        let layout = self.layout.as_ref()?;
        Some(self.view_bounds_for(layout, area))
    }

    pub fn view_bounds_for(&self, layout: &GraphLayout, area: Rect) -> GraphBounds {
        let width = layout.width.max(1.0);
        let height = layout.height.max(1.0);

        let screen_w = area.width as f64;
        let screen_h = area.height.max(1) as f64;
        let aspect_ratio = screen_w / (screen_h * 2.1);

        let view_h = height / self.zoom.max(0.1);
        let view_w = view_h * aspect_ratio;

        let mut center_x = width / 2.0 + self.pan_x;
        let mut center_y = height / 2.0 + self.pan_y;

        if view_w < width {
            let half = view_w / 2.0;
            center_x = center_x.clamp(half, width - half);
        }

        if view_h < height {
            let half = view_h / 2.0;
            center_y = center_y.clamp(half, height - half);
        }

        let x_min = center_x - view_w / 2.0;
        let x_max = center_x + view_w / 2.0;
        let y_min = center_y - view_h / 2.0;
        let y_max = center_y + view_h / 2.0;

        GraphBounds {
            x_min,
            x_max,
            y_min,
            y_max,
        }
    }

    pub fn pan_step(&self, layout: &GraphLayout, area: Rect) -> (f64, f64) {
        let bounds = self.view_bounds_for(layout, area);
        let step_x = (bounds.x_max - bounds.x_min) * 0.1;
        let step_y = (bounds.y_max - bounds.y_min) * 0.1;
        (step_x.max(0.1), step_y.max(0.1))
    }
}
