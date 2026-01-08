//! Layout resolution entry point for rendering.

use std::sync::{Arc, RwLock};

use ratatui::layout::Rect;

use crate::layout::{GridLayout, GridResolver, GridSpec, LayoutPolicy};

/// Shared layout renderer that applies policy overrides before resolving.
///
/// # Examples
/// ```rust
/// use ratatui::layout::Rect;
/// use rotappo_ui_tui::layout::{GridSpec, GridSlot, LayoutPolicy, LayoutRenderer, TrackSize};
///
/// let spec = GridSpec::new(vec![TrackSize::Fill(1)], vec![TrackSize::Fill(1)])
///     .with_slots(vec![GridSlot::new("main", 0, 0)]);
/// let renderer = LayoutRenderer::new(spec, LayoutPolicy::new());
/// let layout = renderer.resolve(Rect::new(0, 0, 10, 5));
/// assert!(layout.rect("main").is_some());
/// ```
#[derive(Clone)]
pub struct LayoutRenderer {
    spec: Arc<RwLock<GridSpec>>,
    policy: LayoutPolicy,
}

impl LayoutRenderer {
    pub fn new(spec: GridSpec, policy: LayoutPolicy) -> Self {
        Self {
            spec: Arc::new(RwLock::new(spec)),
            policy,
        }
    }

    pub fn update_spec(&self, spec: GridSpec) {
        if let Ok(mut guard) = self.spec.write() {
            *guard = spec;
        }
    }

    pub fn resolve(&self, area: Rect) -> GridLayout {
        let spec = match self.spec.read() {
            Ok(guard) => guard.clone(),
            Err(_) => return GridLayout {
                area,
                rects: std::collections::HashMap::new(),
                resolved_at: std::time::Instant::now(),
            },
        };
        let resolved_spec = self.policy.apply(&spec, area);
        GridResolver::resolve(area, &resolved_spec)
    }

    pub async fn resolve_async(&self, area: Rect) -> GridLayout {
        let renderer = self.clone();
        let task = tokio::task::spawn_blocking(move || renderer.resolve(area));
        match task.await {
            Ok(layout) => layout,
            Err(_) => GridLayout {
                area,
                rects: std::collections::HashMap::new(),
                resolved_at: std::time::Instant::now(),
            },
        }
    }

    pub fn policy(&self) -> LayoutPolicy {
        self.policy.clone()
    }
}
