//! Graph rendering support for topology views.

use anyhow::{Context, Result};
use graphviz_rust::cmd::{CommandArg, Format, Layout};
use graphviz_rust::printer::PrinterContext;
use graphviz_rust::{exec, parse};
use ratatui::layout::Rect;
use std::collections::hash_map::DefaultHasher;
use std::collections::{HashMap, HashSet};
use std::env;
use std::hash::{Hash, Hasher};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerminalImageProtocol {
    Kitty,
    ITerm2,
    None,
}

impl TerminalImageProtocol {
    pub fn detect() -> Self {
        if let Some(protocol) = Self::from_env() {
            return protocol;
        }
        if env::var("KITTY_WINDOW_ID").is_ok()
            || env::var("TERM")
                .map(|term| term.contains("kitty"))
                .unwrap_or(false)
        {
            return Self::Kitty;
        }
        if env::var("ITERM_SESSION_ID").is_ok()
            || env::var("TERM_PROGRAM")
                .map(|term| term == "iTerm.app")
                .unwrap_or(false)
        {
            return Self::ITerm2;
        }
        // Default to None if detection fails, to allow ASCII fallback
        Self::None
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Kitty => "Kitty",
            Self::ITerm2 => "iTerm2",
            Self::None => "none",
        }
    }

    fn from_env() -> Option<Self> {
        let value = env::var("ROTAPPO_TUI_GRAPHICS").ok()?;
        match value.to_lowercase().as_str() {
            "kitty" => Some(Self::Kitty),
            "iterm" | "iterm2" | "iterm.app" => Some(Self::ITerm2),
            "none" | "off" | "disabled" => Some(Self::None),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GraphRenderStatus {
    Idle,
    Pending,
    Rendered,
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GraphDirection {
    Left,
    Right,
    Up,
    Down,
}

#[derive(Debug, Clone)]
pub struct GraphRenderRequest {
    pub area: Rect,
    pub dot: String,
}

#[derive(Debug, Clone)]
pub struct GraphNode {
    pub id: String,
    pub label: String,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

#[derive(Debug, Clone)]
pub struct GraphEdge {
    pub tail: usize,
    pub head: usize,
    pub points: Vec<(f64, f64)>,
}

#[derive(Debug, Clone)]
pub struct GraphLayout {
    pub width: f64,
    pub height: f64,
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
    node_index: HashMap<String, usize>,
    outgoing: Vec<Vec<usize>>,
    incoming: Vec<Vec<usize>>,
}

impl GraphLayout {
    pub fn node(&self, id: &str) -> Option<&GraphNode> {
        let index = self.node_index.get(id)?;
        self.nodes.get(*index)
    }

    pub fn node_index(&self, id: &str) -> Option<usize> {
        self.node_index.get(id).copied()
    }

    pub fn dependency_paths(&self, selected_id: &str) -> GraphDependencyPath {
        let Some(selected_index) = self.node_index(selected_id) else {
            return GraphDependencyPath::default();
        };
        let mut nodes = HashSet::new();
        let mut edges = HashSet::new();
        nodes.insert(selected_index);

        let mut stack = vec![selected_index];
        while let Some(index) = stack.pop() {
            for &edge_index in self.outgoing.get(index).into_iter().flatten() {
                if edges.insert(edge_index) {
                    let head = self.edges[edge_index].head;
                    if nodes.insert(head) {
                        stack.push(head);
                    }
                }
            }
        }

        let mut stack = vec![selected_index];
        while let Some(index) = stack.pop() {
            for &edge_index in self.incoming.get(index).into_iter().flatten() {
                if edges.insert(edge_index) {
                    let tail = self.edges[edge_index].tail;
                    if nodes.insert(tail) {
                        stack.push(tail);
                    }
                }
            }
        }

        GraphDependencyPath { nodes, edges }
    }
}

#[derive(Debug, Default, Clone)]
pub struct GraphDependencyPath {
    pub nodes: HashSet<usize>,
    pub edges: HashSet<usize>,
}

#[derive(Debug)]
pub struct GraphRenderState {
    protocol: TerminalImageProtocol,
    request: Option<GraphRenderRequest>,
    cache_hash: Option<u64>,
    image: Option<Vec<u8>>,
    status: GraphRenderStatus,
    error: Option<String>,
    failed_hash: Option<u64>,
    image_id: u32,
    image_active: bool,
    layout: Option<GraphLayout>,
    layout_hash: Option<u64>,
    layout_error: Option<String>,
    selected_id: Option<String>,
    zoom: f64,
    pan_x: f64,
    pan_y: f64,
}

impl GraphRenderState {
    pub fn new() -> Self {
        Self {
            protocol: TerminalImageProtocol::detect(),
            request: None,
            cache_hash: None,
            image: None,
            status: GraphRenderStatus::Idle,
            error: None,
            failed_hash: None,
            image_id: 1,
            image_active: false,
            layout: None,
            layout_hash: None,
            layout_error: None,
            selected_id: None,
            zoom: 1.0, // Initial zoom
            pan_x: 0.0,
            pan_y: 0.0,
        }
    }

    // Helper to determine if we should fit-to-screen on first load
    // For now we trust the layout/viewport logic to center it.
    // The viewport logic centers on (width/2 + pan_x, height/2 + pan_y)
    // with width = full_width / zoom.
    // So zoom=1.0 shows the full graph if we don't scale?
    // Wait, if zoom=1.0, view_w = full_width. So it shows the full graph.
    // This is EXACTLY what we want for "massive" graphs - show the whole thing first?
    // OR if it's too detailed, maybe zooming in is better?
    // TUI usually benefits from "Overview first, zoom and filter details on demand".
    // So zoom=1.0 showing full graph is correct default.
    // The "massive" complaint might be that it's physically huge image rendered?
    // With -Gviewport, we render a crop/scaled version.
    // But -Gviewport with Z=1 and W=full_width might still produce a huge image if full_width is large.
    // Wait, -Gviewport defines the *input window*.
    // The *output size* is determined by -Gsize or dpi.
    // If we don't constrain output size, Graphviz outputs natural size.
    // If the natural size of the full graph is 10000x10000, we get a huge PNG.
    // runner.rs scales it down to the terminal.
    // So the user sees a tiny, unreadable mess.
    // To navigate, they zoom in.
    // If they zoom in (zoom=2.0), view_w = full_width/2.
    // We render half the graph. Natural size is still large?
    // We should potentially combine -Gviewport with -Gsize or -Gdpi to control quality/performance.
    // But for now, let's stick to viewport injection.

    pub fn protocol(&self) -> TerminalImageProtocol {
        self.protocol
    }

    pub fn supports_images(&self) -> bool {
        matches!(
            self.protocol,
            TerminalImageProtocol::Kitty | TerminalImageProtocol::ITerm2
        )
    }

    pub fn protocol_label(&self) -> &'static str {
        self.protocol.label()
    }

    pub fn status(&self) -> GraphRenderStatus {
        self.status
    }

    pub fn last_error(&self) -> Option<&str> {
        self.error.as_deref()
    }

    pub fn request(&self) -> Option<&GraphRenderRequest> {
        self.request.as_ref()
    }

    pub fn image(&self) -> Option<&[u8]> {
        self.image.as_deref()
    }

    pub fn image_id(&self) -> u32 {
        self.image_id
    }

    pub fn image_active(&self) -> bool {
        self.image_active
    }

    pub fn set_image_active(&mut self, active: bool) {
        self.image_active = active;
    }

    pub fn clear_request(&mut self) {
        self.request = None;
    }

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

        // Calculate view-dependent hash
        let mut hasher = DefaultHasher::new();
        request.dot.hash(&mut hasher);
        // Include viewport params in hash
        format!("{:.2},{:.2},{:.2}", self.zoom, self.pan_x, self.pan_y).hash(&mut hasher);
        // Also hash the area size, because if terminal resizes, we need new image size
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

        // Calculate target size in inches for Graphviz
        // Assume rough cell size 10x20 pixels? Or just use "sufficiently large" pixels?
        // Graphviz uses 96 DPI by default.
        // Let's assume 1 cell = 0.1 x 0.2 inches? (10px x 20px approx)
        // This is a heuristic.
        // Or simpler: Width in cells / 2 = Width in inches? (80 cells -> 40 inches? No too big).
        // Standard terminal is 80x24.
        // Let's say 80 cells is ~8 inches? (10 chars per inch).
        // 24 lines is ~4 inches? (6 lines per inch).
        // So: width / 10.0, height / 6.0.
        let target_w = (request.area.width as f64) / 10.0;
        let target_h = (request.area.height as f64) / 5.0; // slightly denser lines

        // Calculate viewport args if we have layout info
        let viewport_arg = if let Some(layout) = self.layout.as_ref() {
            // Let's use view_bounds_for to get the rect in graph coords (inches).
            let b = self.view_bounds_for(layout, request.area);

            // Add 5% padding to viewport to prevent label clipping at edges
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
            // First render (no layout yet), use default (fit all?)
            // Or maybe just let it render full size once to parse layout?
            // Actually we call ensure_layout separately.
            None
        };

        // Note: ensure_layout must be called before ensure_image for viewport to work effectively
        // but ensure_layout uses `plain` render which is fast.

        let mut args = vec![
            CommandArg::Format(Format::Png),
            CommandArg::Layout(Layout::Dot),
        ];

        // Enforce output size to prevent massive images
        // "!": force dimensions (don't scale implicitly to fit, but we want it to fit? No, we want it to BE this size).
        // actually just "W,H" tells graphviz "fit graph into this box".
        // combined with viewport, it tells "fit VIEWPORT into this box".
        args.push(CommandArg::Custom(format!(
            "-Gsize={target_w:.2},{target_h:.2}!"
        )));

        // Improve Layout / Aesthetics
        // -Goverlap=false: prismic overlap removal (prevents node overlaps)
        // -Gsplines=true: draw edges as splines (avoiding nodes)
        // -Gnodesep=0.6: min space between nodes (default 0.25)
        // -Granksep=0.8: min space between ranks (default 0.5)
        // -Gdpi=144: Higher DPI for sharper text (standard is 96).
        //            Note: Higher DPI + Fixed Size (inches) = More Pixels.
        //            The terminal image protocol will scale it down to cell area anyway,
        //            but downsampling from high-res looks better than upsampling or 1:1 matching sometimes.

        args.push(CommandArg::Custom("-Goverlap=false".to_string()));
        args.push(CommandArg::Custom("-Gsplines=true".to_string()));
        args.push(CommandArg::Custom("-Gnodesep=0.6".to_string()));
        args.push(CommandArg::Custom("-Granksep=1.0".to_string()));
        // Increase DPI slightly for readability?
        // args.push(CommandArg::Custom("-Gdpi=120".to_string()));

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
        // ... (rest is same, but we need to reconstruct hash logic inside mark_failed if we change it above)
        // Simplification: We'll calculate hash again or store it in queue_request?
        // Actually queue_request doesn't know zoom yet for the *next* render.
        // Let's just use 0 or simple hash for failure to avoid complexity, or recalculate.
        let mut hasher = DefaultHasher::new();
        if let Some(req) = &self.request {
            req.dot.hash(&mut hasher);
            format!("{:.2},{:.2},{:.2}", self.zoom, self.pan_x, self.pan_y).hash(&mut hasher);
            self.failed_hash = Some(hasher.finish());
        }
        self.status = GraphRenderStatus::Failed;
        self.error = Some(error);
    }

    pub fn ensure_layout(&mut self, dot: &str) -> Result<()> {
        let hash = hash_dot(dot);
        if self.layout_hash == Some(hash) {
            return Ok(());
        }
        let plain = render_dot_plain(dot).context("graphviz plain render failed")?;
        let layout = parse_plain_layout(&plain).context("graphviz plain parse failed")?;
        let previous = self.selected_id.clone();
        self.selected_id = previous
            .filter(|id| layout.node_index.contains_key(id))
            .or_else(|| layout.nodes.first().map(|node| node.id.clone()));
        self.layout = Some(layout);
        self.layout_hash = Some(hash);
        self.layout_error = None;
        Ok(())
    }

    pub fn layout(&self) -> Option<&GraphLayout> {
        self.layout.as_ref()
    }

    pub fn layout_error(&self) -> Option<&str> {
        self.layout_error.as_deref()
    }

    pub fn mark_layout_failed(&mut self, error: String) {
        self.layout_error = Some(error);
        self.layout = None;
        self.layout_hash = None;
    }

    pub fn selected_id(&self) -> Option<&str> {
        self.selected_id.as_deref()
    }

    pub fn selected_node(&self) -> Option<&GraphNode> {
        let id = self.selected_id.as_deref()?;
        self.layout.as_ref()?.node(id)
    }

    pub fn select_node(&mut self, id: &str) -> bool {
        if self.selected_id.as_deref() == Some(id) {
            return false;
        }
        self.selected_id = Some(id.to_string());

        // Auto-pan to selected node (Camera Follow)
        // logic: view_center = graph_center + pan
        // we want view_center = node_center
        // so pan = node_center - graph_center
        if let Some(layout) = self.layout.as_ref() {
            if let Some(node) = layout.node(id) {
                let graph_center_x = layout.width / 2.0;
                let graph_center_y = layout.height / 2.0;
                self.pan_x = node.x - graph_center_x;
                self.pan_y = node.y - graph_center_y;
            }
        }
        true
    }

    pub fn node_id_at(&self, x: f64, y: f64) -> Option<String> {
        let layout = self.layout.as_ref()?;

        // Strict containment first
        let exact = layout
            .nodes
            .iter()
            .find(|node| {
                let half_w = node.width / 2.0;
                let half_h = node.height / 2.0;
                x >= node.x - half_w
                    && x <= node.x + half_w
                    && y >= node.y - half_h
                    && y <= node.y + half_h
            })
            .map(|n| n.id.clone());

        if exact.is_some() {
            return exact;
        }

        // Fuzzy search
        // User requested "neighborhood" trigger - increasing radius to 3.0 (approx 3 inches/30 chars)
        let radius = 3.0;
        layout
            .nodes
            .iter()
            .filter_map(|node| {
                let dx = x - node.x;
                let dy = y - node.y;
                let dist = (dx * dx + dy * dy).sqrt();
                if dist < radius {
                    Some((dist, node.id.clone()))
                } else {
                    None
                }
            })
            .min_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(_, id)| id)
    }

    pub fn select_node_at(&mut self, x: f64, y: f64) -> bool {
        let Some(layout) = self.layout.as_ref() else {
            return false;
        };

        // First check strict containment (higher priority)
        // First check strict containment (higher priority)
        let exact_match = layout
            .nodes
            .iter()
            .find(|node| {
                let half_w = node.width / 2.0;
                let half_h = node.height / 2.0;
                x >= node.x - half_w
                    && x <= node.x + half_w
                    && y >= node.y - half_h
                    && y <= node.y + half_h
            })
            .map(|n| n.id.clone());

        if let Some(id) = exact_match {
            return self.select_node(&id);
        }

        // Fuzzy search: closest node within defined radius (e.g. 0.5 inches)
        let radius = 0.5;
        let best_match = layout
            .nodes
            .iter()
            .filter_map(|node| {
                let dx = x - node.x;
                let dy = y - node.y;
                let dist = (dx * dx + dy * dy).sqrt();
                if dist < radius {
                    Some((dist, node.id.clone()))
                } else {
                    None
                }
            })
            .min_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(_, id)| id);

        if let Some(id) = best_match {
            self.select_node(&id)
        } else {
            false
        }
    }

    pub fn select_next(&mut self) -> bool {
        let next_id = {
            let Some(layout) = self.layout.as_ref() else {
                return false;
            };
            if layout.nodes.is_empty() {
                return false;
            }
            let next_index = match self.selected_id.as_deref() {
                Some(id) => layout
                    .node_index(id)
                    .map(|index| (index + 1) % layout.nodes.len())
                    .unwrap_or(0),
                None => 0,
            };
            layout.nodes[next_index].id.clone()
        };
        self.select_node(&next_id)
    }

    pub fn select_prev(&mut self) -> bool {
        let prev_id = {
            let Some(layout) = self.layout.as_ref() else {
                return false;
            };
            if layout.nodes.is_empty() {
                return false;
            }
            let prev_index = match self.selected_id.as_deref() {
                Some(id) => layout
                    .node_index(id)
                    .map(|index| (index + layout.nodes.len() - 1) % layout.nodes.len())
                    .unwrap_or(0),
                None => 0,
            };
            layout.nodes[prev_index].id.clone()
        };
        self.select_node(&prev_id)
    }

    pub fn select_direction(&mut self, direction: GraphDirection) -> bool {
        let best_id = {
            let Some(layout) = self.layout.as_ref() else {
                return false;
            };
            let Some(current_id) = self.selected_id.as_deref() else {
                // Cannot call mutable select_next here either if we hold layout borrow?
                // Actually select_next handles its ownborrow.
                // But we are inside the scope of the layout borrow.
                // We should return None here and handle it outside.
                return self.select_next();
            };
            let Some(current) = layout.node(current_id) else {
                return false;
            };
            let dir = match direction {
                GraphDirection::Left => (-1.0, 0.0),
                GraphDirection::Right => (1.0, 0.0),
                GraphDirection::Up => (0.0, 1.0),
                GraphDirection::Down => (0.0, -1.0),
            };
            let mut best = None;
            for node in &layout.nodes {
                if node.id == current.id {
                    continue;
                }
                let dx = node.x - current.x;
                let dy = node.y - current.y;
                let dist = (dx * dx + dy * dy).sqrt();
                if dist == 0.0 {
                    continue;
                }
                let alignment = (dx * dir.0 + dy * dir.1) / dist;
                if alignment < 0.3 {
                    continue;
                }
                let score = (1.0 - alignment) * 10.0 + dist;
                if best
                    .as_ref()
                    .map(|(_, best_score)| score < *best_score)
                    .unwrap_or(true)
                {
                    best = Some((node.id.clone(), score));
                }
            }
            best.map(|(id, _)| id)
        };

        if let Some(id) = best_id {
            self.select_node(&id)
        } else {
            false
        }
    }

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

        // Calculate target aspect ratio from screen area
        // Assuming roughly 1:2 char aspect ratio for terminal cells if not using specialized font?
        // Actually, Graphviz -Gsize uses INCHES.
        // We want the viewport shape to match the physical screen shape.
        // A ratatui Rect area has W columns and H rows.
        // If we assume a square font (nerd font often is), W/H is aspect.
        // But usually H is 2x W physically.
        // Let's assume standard terminal: 1 cell = 0.5 aspect (width is half of height).
        // So physical width = W * 1. Physical height = H * 2.
        // Aspect = (W * 1) / (H * 2) = W / (2H).
        // Let's try 0.5 correction factor.
        let screen_w = area.width as f64;
        let screen_h = area.height.max(1) as f64;
        // Correction factor: Width is "units", Height is "units".
        // If 100x100 chars -> usually physically wider than tall? No, taller.
        // No, 80x24 is 4:3 roughly?
        // 80 / (24*2) = 80/48 = 1.666 = 16:9 roughly.
        // So factor is 2.0 (height is 2x density).
        // aspect = w / (h * 2.0).  (assuming height is pixels/2)
        // Actually simplest is just use W/H ratio and let user adjust if needed?
        // Or assume 2.0 factor.
        let aspect_ratio = screen_w / (screen_h * 2.1); // 2.1 heuristic for typical fonts

        // Determine view dimensions based on zoom
        // We define Zoom=1.0 as "Fit Height to Screen".
        let view_h = height / self.zoom.max(0.1);
        let view_w = view_h * aspect_ratio;

        let mut center_x = width / 2.0 + self.pan_x;
        let mut center_y = height / 2.0 + self.pan_y;

        // Clamp center to valid range (keep at least some overlap)
        if view_w >= width {
            // If view is wider than graph, just center it
            // actually allow panning a bit?
            // let's just stick to graph center horizontal if zooming out far
            // center_x = width / 2.0;
        } else {
            let half = view_w / 2.0;
            center_x = center_x.clamp(half, width - half);
        }

        if view_h >= height {
            // center_y = height / 2.0;
        } else {
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

#[derive(Debug, Clone, Copy)]
pub struct GraphBounds {
    pub x_min: f64,
    pub x_max: f64,
    pub y_min: f64,
    pub y_max: f64,
}

fn hash_dot(dot: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    dot.hash(&mut hasher);
    hasher.finish()
}

fn render_dot_with_args(dot: &str, args: Vec<CommandArg>) -> Result<Vec<u8>> {
    let graph = parse(dot).map_err(|e| anyhow::anyhow!("failed to parse DOT: {e}"))?;
    let bytes =
        exec(graph, &mut PrinterContext::default(), args).context("failed to execute graphviz")?;
    Ok(bytes)
}

fn render_dot_plain(dot: &str) -> Result<String> {
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
    use super::*;

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
        // Force protocol to Kitty to verify image generation path
        state.protocol = TerminalImageProtocol::Kitty;

        let dot = "digraph G { a -> b; }";
        // Mock a request area
        state.queue_request(Rect::new(0, 0, 100, 100), dot.to_string());

        let res = state.ensure_image();
        assert!(res.is_ok(), "ensure_image failed: {:?}", res.err());

        // Assert image is generated
        assert!(state.image().is_some(), "Image bytes should be present");
        assert!(
            state.image().unwrap().len() > 0,
            "Image should not be empty"
        );
        assert_eq!(state.status(), GraphRenderStatus::Rendered);
    }
}

#[derive(Debug)]
struct GraphEdgeRaw {
    tail: String,
    head: String,
    points: Vec<(f64, f64)>,
}

fn parse_plain_layout(text: &str) -> Result<GraphLayout> {
    let mut width = 0.0;
    let mut height = 0.0;
    let mut nodes = Vec::new();
    let mut edges = Vec::new();

    for line in text.lines() {
        let tokens = if line.contains('"') {
            // Basic quote handling if needed, though simple split usually works for plain
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

    let mut node_index = HashMap::new();
    for (idx, node) in nodes.iter().enumerate() {
        node_index.insert(node.id.clone(), idx);
    }

    let mut outgoing = vec![Vec::new(); nodes.len()];
    let mut incoming = vec![Vec::new(); nodes.len()];
    let mut resolved_edges = Vec::new();
    for edge in edges {
        let Some(&tail) = node_index.get(&edge.tail) else {
            continue;
        };
        let Some(&head) = node_index.get(&edge.head) else {
            continue;
        };
        let edge_index = resolved_edges.len();
        resolved_edges.push(GraphEdge {
            tail,
            head,
            points: edge.points,
        });
        outgoing[tail].push(edge_index);
        incoming[head].push(edge_index);
    }

    Ok(GraphLayout {
        width,
        height,
        nodes,
        edges: resolved_edges,
        node_index,
        outgoing,
        incoming,
    })
}

fn split_plain_tokens(line: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;
    let mut chars = line.chars().peekable();
    while let Some(ch) = chars.next() {
        if in_quotes {
            if ch == '\\' {
                if let Some(next) = chars.next() {
                    current.push(next);
                }
                continue;
            }
            if ch == '"' {
                in_quotes = false;
                continue;
            }
            current.push(ch);
            continue;
        }
        if ch == '"' {
            in_quotes = true;
            continue;
        }
        if ch.is_whitespace() {
            if !current.is_empty() {
                tokens.push(current.clone());
                current.clear();
            }
            continue;
        }
        current.push(ch);
    }
    if !current.is_empty() {
        tokens.push(current);
    }
    tokens
}
