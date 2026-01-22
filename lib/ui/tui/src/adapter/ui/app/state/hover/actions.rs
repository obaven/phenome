use ratatui::layout::Margin;

use crate::app::App;

pub(super) fn hover_index_in_actions(app: &App, row: u16) -> Option<usize> {
    let margin = if matches!(app.active_view(), crate::app::NavView::TerminalCommands) {
        Margin {
            horizontal: 0,
            vertical: 0,
        }
    } else {
        Margin {
            horizontal: 1,
            vertical: 1,
        }
    };
    let inner = app.ui.actions_area.inner(margin);
    if inner.height == 0 || row < inner.y || row >= inner.y + inner.height {
        return None;
    }
    let offset = row.saturating_sub(inner.y) as usize;
    let item_height = if matches!(app.active_view(), crate::app::NavView::TerminalCommands) {
        1usize
    } else {
        2usize
    };
    let index = offset / item_height + app.ui.actions_scroll as usize;
    if index < app.runtime.registry().actions().len() {
        Some(index)
    } else {
        None
    }
}
