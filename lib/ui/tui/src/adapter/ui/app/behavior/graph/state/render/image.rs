use anyhow::{Context, Result};
use graphviz_rust::cmd::{CommandArg, Format, Layout};
use ratatui::layout::Rect;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use super::super::core::GraphRenderState;
use super::super::super::render::{hash_dot, render_dot_with_args};
use super::super::super::types::{GraphRenderRequest, GraphRenderStatus};

impl GraphRenderState {
    pub fn queue_request(&mut self, area: Rect, dot: String) {
        let hash = hash_dot(&dot);
        self.request = Some(GraphRenderRequest { area, dot });
        if !self.supports_images() {
            self.status = GraphRenderStatus::Idle;
            return;
        }
        if self.cache_hash == Some(hash) {
            self.status = GraphRenderStatus::Rendered;
            return;
        }
        if self.failed_hash == Some(hash) {
            self.status = GraphRenderStatus::Failed;
            return;
        }
        self.status = GraphRenderStatus::Pending;
    }

    pub fn ensure_image(&mut self) -> Result<()> {
        let request = match self.request.as_ref() {
            Some(request) => request,
            None => {
                self.status = GraphRenderStatus::Idle;
                return Ok(());
            }
        };
        if !self.supports_images() {
            self.status = GraphRenderStatus::Idle;
            return Ok(());
        }

        let mut hasher = DefaultHasher::new();
        request.dot.hash(&mut hasher);
        format!("{:.2},{:.2},{:.2}", self.zoom, self.pan_x, self.pan_y).hash(&mut hasher);
        request.area.width.hash(&mut hasher);
        request.area.height.hash(&mut hasher);
        let hash = hasher.finish();

        if self.cache_hash == Some(hash) {
            self.status = GraphRenderStatus::Rendered;
            return Ok(());
        }
        if self.failed_hash == Some(hash) {
            self.status = GraphRenderStatus::Failed;
            return Ok(());
        }

        let target_w = (request.area.width as f64) / 10.0;
        let target_h = (request.area.height as f64) / 5.0;

        let viewport_arg = if let Some(layout) = self.layout.as_ref() {
            let b = self.view_bounds_for(layout, request.area);
            let pad_w = (b.x_max - b.x_min) * 0.05;
            let pad_h = (b.y_max - b.y_min) * 0.05;
            let width = (b.x_max - b.x_min) + pad_w * 2.0;
            let height = (b.y_max - b.y_min) + pad_h * 2.0;
            let center_x = (b.x_max + b.x_min) / 2.0;
            let center_y = (b.y_max + b.y_min) / 2.0;
            Some(format!(
                "{width:.3},{height:.3},1,{center_x:.3},{center_y:.3}"
            ))
        } else {
            None
        };

        let mut args = vec![
            CommandArg::Format(Format::Png),
            CommandArg::Layout(Layout::Dot),
        ];
        args.push(CommandArg::Custom(format!(
            "-Gsize={target_w:.2},{target_h:.2}!"
        )));
        args.push(CommandArg::Custom("-Goverlap=false".to_string()));
        args.push(CommandArg::Custom("-Gsplines=true".to_string()));
        args.push(CommandArg::Custom("-Gnodesep=0.6".to_string()));
        args.push(CommandArg::Custom("-Granksep=1.0".to_string()));
        if let Some(vp) = viewport_arg {
            args.push(CommandArg::Custom(format!("-Gviewport={vp}")));
        }

        let png = render_dot_with_args(&request.dot, args).context("graphviz render failed")?;
        self.cache_hash = Some(hash);
        self.image = Some(png);
        self.status = GraphRenderStatus::Rendered;
        self.error = None;
        Ok(())
    }

    pub fn mark_failed(&mut self, error: String) {
        let mut hasher = DefaultHasher::new();
        if let Some(req) = &self.request {
            req.dot.hash(&mut hasher);
            format!("{:.2},{:.2},{:.2}", self.zoom, self.pan_x, self.pan_y).hash(&mut hasher);
            self.failed_hash = Some(hasher.finish());
        }
        self.status = GraphRenderStatus::Failed;
        self.error = Some(error);
    }
}
