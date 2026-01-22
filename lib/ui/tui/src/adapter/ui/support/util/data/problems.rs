//! Problem collection helpers.

use phenome_ui_presentation::formatting;

/// Gather formatted problem lines from the current runtime state.
pub fn collect_problems(app: &crate::app::App) -> Vec<String> {
    let health = app.context.ports.health.snapshot();
    formatting::problem_lines(app.runtime.snapshot(), Some(&health))
}
