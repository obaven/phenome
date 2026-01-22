//! Terminal setup and shared event loop helpers.

use anyhow::Result;
use crossterm::{
    event::{self, Event as CrosstermEvent},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::backend::CrosstermBackend;
use ratatui::prelude::Frame;
use ratatui::Terminal;
use std::io::{self, Stdout};
use std::time::Duration;

/// Guard for raw mode + alternate screen that restores terminal state on drop.
pub(crate) struct TerminalGuard {
    terminal: Terminal<CrosstermBackend<Stdout>>,
}

impl TerminalGuard {
    pub(crate) fn new() -> Result<Self> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, crossterm::event::EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;
        terminal.clear()?;
        Ok(Self { terminal })
    }

    pub(crate) fn terminal_mut(&mut self) -> &mut Terminal<CrosstermBackend<Stdout>> {
        &mut self.terminal
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(
            self.terminal.backend_mut(),
            LeaveAlternateScreen,
            crossterm::event::DisableMouseCapture
        );
        let _ = self.terminal.show_cursor();
    }
}

pub(crate) fn run_tui_loop<T, FRender, FAfter, FEvent, FTick, FQuit>(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    tick_rate: Duration,
    app: &mut T,
    mut render: FRender,
    mut after_draw: FAfter,
    mut handle_event: FEvent,
    mut on_tick: FTick,
    mut should_quit: FQuit,
) -> Result<()>
where
    FRender: FnMut(&mut Frame, &mut T),
    FAfter: FnMut(&mut Terminal<CrosstermBackend<Stdout>>, &mut T) -> Result<()>,
    FEvent: FnMut(CrosstermEvent, &mut T) -> Result<()>,
    FTick: FnMut(&mut T),
    FQuit: FnMut(&T) -> bool,
{
    loop {
        terminal.draw(|frame| render(frame, app))?;
        after_draw(terminal, app)?;
        if should_quit(app) {
            break;
        }
        if event::poll(tick_rate)? {
            let event = event::read()?;
            handle_event(event, app)?;
        }
        on_tick(app);
    }
    Ok(())
}
