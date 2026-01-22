//! Terminal setup and event loop wiring for the TUI.

use anyhow::Result;
use crossterm::event::Event as CrosstermEvent;
use std::time::Duration;

use crate::app::{App, AppContext};
use crate::terminal::{TerminalGuard, run_tui_loop};
use phenome_application::Runtime;

use super::render::render;

mod graph;
mod iterm;
mod kitty;

/// Launch the TUI and enter the event loop.
pub fn start(runtime: Runtime, context: AppContext) -> Result<()> {
    let mut terminal_guard = TerminalGuard::new()?;
    let mut app = App::new(runtime, context);
    let is_tmux = std::env::var("TMUX").is_ok()
        || std::env::var("TERM")
            .map(|t| t.starts_with("screen") || t.starts_with("tmux"))
            .unwrap_or(false);
    let tick_rate = Duration::from_millis(200);
    run_tui_loop(
        terminal_guard.terminal_mut(),
        tick_rate,
        &mut app,
        |frame, app| render(frame, app),
        |terminal, app| graph::render_graph(terminal, app, is_tmux),
        |event, app| {
            match event {
                CrosstermEvent::Key(key) => app.handle_key_event(key)?,
                CrosstermEvent::Mouse(mouse) => app.handle_mouse_event(mouse)?,
                _ => {}
            }
            Ok(())
        },
        |app| app.on_tick(),
        |app| app.should_quit,
    )
}
