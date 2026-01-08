//! Cached grid layouts for resize-heavy rendering.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::Instant;

use ratatui::layout::Rect;

use super::{GridLayout, GridResolver, GridSpec};

/// Thread-safe cache for resolved grid layouts.
///
/// # Examples
/// ```rust
/// use ratatui::layout::Rect;
/// use rotappo_ui_tui::layout::{GridCache, GridSpec, GridSlot, TrackSize};
///
/// let spec = GridSpec::new(vec![TrackSize::Fill(1)], vec![TrackSize::Fill(1)])
///     .with_slots(vec![GridSlot::new("main", 0, 0)]);
/// let cache = GridCache::new(spec);
/// let layout = cache.resolve(Rect::new(0, 0, 10, 5));
/// assert!(layout.rect("main").is_some());
/// ```
#[derive(Clone)]
pub struct GridCache {
    inner: Arc<RwLock<GridCacheState>>,
}

impl GridCache {
    /// Create a new cache for the given grid spec.
    pub fn new(spec: GridSpec) -> Self {
        let layout = GridResolver::resolve(Rect::default(), &spec);
        Self {
            inner: Arc::new(RwLock::new(GridCacheState {
                spec,
                layout,
                last_area: Rect::default(),
            })),
        }
    }

    /// Replace the cached spec and invalidate stored layout.
    pub fn update_spec(&self, spec: GridSpec) {
        if let Ok(mut guard) = self.inner.write() {
            guard.spec = spec;
            guard.last_area = Rect::default();
        }
    }

    /// Resolve a layout and update the cache if needed.
    pub fn resolve(&self, area: Rect) -> GridLayout {
        if let Ok(mut guard) = self.inner.write() {
            if guard.last_area != area {
                guard.layout = GridResolver::resolve(area, &guard.spec);
                guard.last_area = area;
            }
            return guard.layout.clone();
        }
        GridLayout {
            area,
            rects: HashMap::new(),
            resolved_at: Instant::now(),
        }
    }

    /// Get the most recent cached layout without resolving.
    pub fn snapshot(&self) -> GridLayout {
        if let Ok(guard) = self.inner.read() {
            return guard.layout.clone();
        }
        GridLayout {
            area: Rect::default(),
            rects: HashMap::new(),
            resolved_at: Instant::now(),
        }
    }

    /// Resolve a layout on a blocking task.
    pub async fn resolve_async(&self, area: Rect) -> GridLayout {
        let cache = self.clone();
        let task = tokio::task::spawn_blocking(move || cache.resolve(area));
        match task.await {
            Ok(layout) => layout,
            Err(_) => self.snapshot(),
        }
    }
}

struct GridCacheState {
    spec: GridSpec,
    layout: GridLayout,
    last_area: Rect,
}
