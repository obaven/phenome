use ratatui::layout::Margin;

use crate::util::assembly_lines;

use crate::app::App;

pub(super) fn hover_index_in_assembly(app: &App, row: u16) -> Option<usize> {
    let inner = app.ui.assembly_area.inner(Margin {
        horizontal: 1,
        vertical: 1,
    });
    if inner.height == 0 || row < inner.y || row >= inner.y + inner.height {
        return None;
    }
    let offset = row.saturating_sub(inner.y) as usize;
    let lines = assembly_lines(app.runtime.snapshot());
    let line_index = offset + app.ui.assembly_scroll as usize;
    lines.get(line_index).and_then(|line| line.step_index)
}
