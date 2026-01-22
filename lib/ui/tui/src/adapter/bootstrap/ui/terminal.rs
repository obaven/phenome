use crate::bootstrap::app::BootstrapApp;
use crate::terminal::run_tui_loop;
use anyhow::Result;
use crossterm::event::Event as CrosstermEvent;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io::Stdout;
use std::time::Duration;

const TICK_RATE: Duration = Duration::from_millis(200);

pub fn run_app(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    app: &mut BootstrapApp,
) -> Result<()> {
    run_tui_loop(
        terminal,
        TICK_RATE,
        app,
        |frame, app| app.render(frame),
        |_terminal, _app| Ok(()),
        |event, app| {
            if let CrosstermEvent::Key(key) = event {
                app.handle_key_event(key)?;
            }
            Ok(())
        },
        |app| app.on_tick(),
        |app| app.should_quit,
    )
}
