use ratatui::layout::Margin;

use crate::app::App;

pub(super) fn hover_index_in_capabilities(app: &App, row: u16) -> Option<usize> {
    let inner = app.ui.capabilities_area.inner(Margin {
        horizontal: 1,
        vertical: 1,
    });
    if inner.height == 0 || row < inner.y || row >= inner.y + inner.height {
        return None;
    }
    let offset = row.saturating_sub(inner.y) as usize;
    let index = offset + app.ui.capabilities_scroll as usize;
    if index < app.runtime.snapshot().capabilities.len() {
        Some(index)
    } else {
        None
    }
}
