//! Terminal setup and event loop wiring for the TUI.

use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event as CrosstermEvent},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::backend::{Backend, CrosstermBackend};
use ratatui::Terminal;
use std::io::{self, Stdout};
use std::time::Duration;

use rotappo_application::Runtime;
use crate::app::{App, AppContext};

use super::render::render;

/// Launch the TUI and enter the event loop.
pub fn start(runtime: Runtime, context: AppContext) -> Result<()> {
    let mut terminal_guard = TerminalGuard::new()?;
    let mut app = App::new(runtime, context);
    run_app(terminal_guard.terminal_mut(), &mut app)
}

struct TerminalGuard {
    terminal: Terminal<CrosstermBackend<Stdout>>,
}

impl TerminalGuard {
    fn new() -> Result<Self> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;
        terminal.clear()?;
        Ok(Self { terminal })
    }

    fn terminal_mut(&mut self) -> &mut Terminal<CrosstermBackend<Stdout>> {
        &mut self.terminal
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(
            self.terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        );
        let _ = self.terminal.show_cursor();
    }
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> Result<()> {
    let tick_rate = Duration::from_millis(200);
    loop {
        terminal.draw(|frame| render(frame, app))?;
        if app.should_quit {
            break;
        }
        if event::poll(tick_rate)? {
            match event::read()? {
                CrosstermEvent::Key(key) => app.handle_key_event(key)?,
                CrosstermEvent::Mouse(mouse) => app.handle_mouse_event(mouse)?,
                _ => {}
            }
        }
        app.on_tick();
    }
    Ok(())
}
