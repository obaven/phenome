use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::prelude::Frame;

use crate::app::App;
use phenome_domain::AssemblyStep;

mod columns;
mod lineage;

pub(super) fn render_assembly_detail(
    frame: &mut Frame,
    area: Rect,
    app: &App,
    step: &AssemblyStep,
) {
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),
            Constraint::Min(1),
        ])
        .split(area);

    lineage::render_lineage(frame, main_chunks[0], app.runtime.snapshot(), step);
    columns::render_columns(frame, main_chunks[1], app, step);
}
