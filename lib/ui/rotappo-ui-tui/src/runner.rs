//! Terminal setup and event loop wiring for the TUI.

use anyhow::Result;
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use crossterm::{
    cursor::MoveTo,
    event::{self, DisableMouseCapture, EnableMouseCapture, Event as CrosstermEvent},
    execute, queue,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use std::io::Write;
use std::io::{self, Stdout};
use std::time::Duration;

use crate::app::{App, AppContext, PanelId, TerminalImageProtocol};
use rotappo_application::Runtime;

use super::render::render;

/// Launch the TUI and enter the event loop.
pub fn start(runtime: Runtime, context: AppContext) -> Result<()> {
    let mut terminal_guard = TerminalGuard::new()?;
    let mut app = App::new(runtime, context);
    let is_tmux = std::env::var("TMUX").is_ok()
        || std::env::var("TERM")
            .map(|t| t.starts_with("screen") || t.starts_with("tmux"))
            .unwrap_or(false);
    run_app(terminal_guard.terminal_mut(), &mut app, is_tmux)
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

fn run_app(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    app: &mut App,
    is_tmux: bool,
) -> Result<()> {
    let tick_rate = Duration::from_millis(200);
    loop {
        // Render graph first so it stays behind UI elements
        terminal.draw(|frame| render(frame, app))?;
        render_graph(terminal, app, is_tmux)?;
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

fn render_graph(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    app: &mut App,
    is_tmux: bool,
) -> Result<()> {
    if !app.graph.supports_images() {
        return Ok(());
    }
    let notifications_open = !app.panel_collapsed(PanelId::Notifications);
    if notifications_open {
        if app.graph.image_active() {
            clear_graph_image(terminal, app)?;
            app.graph.set_image_active(false);
        }
        return Ok(());
    }
    let Some(request) = app.graph.request().cloned() else {
        if app.graph.image_active() {
            clear_graph_image(terminal, app)?;
            app.graph.set_image_active(false);
        }
        return Ok(());
    };
    if request.area.width < 2 || request.area.height < 2 {
        return Ok(());
    }
    if let Err(err) = app.graph.ensure_image() {
        app.graph.mark_failed(err.to_string());
        return Ok(());
    }
    let Some(image) = app.graph.image() else {
        return Ok(());
    };

    let stdout = terminal.backend_mut();
    queue!(stdout, MoveTo(request.area.x, request.area.y))?;
    match app.graph.protocol() {
        TerminalImageProtocol::Kitty => {
            write_kitty_image(stdout, image, request.area, app.graph.image_id(), is_tmux)?
        }
        TerminalImageProtocol::ITerm2 => write_iterm2_image(stdout, image, request.area)?,
        TerminalImageProtocol::None => {}
    }
    stdout.flush()?;
    app.graph.set_image_active(true);
    Ok(())
}

fn clear_graph_image(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    app: &mut App,
) -> Result<()> {
    match app.graph.protocol() {
        TerminalImageProtocol::Kitty => {
            let stdout = terminal.backend_mut();
            write!(stdout, "\x1b_Ga=d,d=A\x1b\\")?;
            stdout.flush()?;
        }
        TerminalImageProtocol::ITerm2 => {
            if let Some(request) = app.graph.request() {
                let stdout = terminal.backend_mut();
                let spaces = " ".repeat(request.area.width as usize);
                for y in 0..request.area.height {
                    queue!(
                        stdout,
                        MoveTo(request.area.x, request.area.y + y),
                        crossterm::style::Print(&spaces)
                    )?;
                }
                stdout.flush()?;
            }
        }
        _ => {}
    }
    Ok(())
}

fn write_kitty_image<W: Write>(
    stdout: &mut W,
    image: &[u8],
    area: ratatui::layout::Rect,
    image_id: u32,
    is_tmux: bool,
) -> Result<()> {
    let encoded = STANDARD.encode(image);
    let chunk_size = 4096;
    let total_chunks = (encoded.len() + chunk_size - 1) / chunk_size;
    for (index, chunk) in encoded.as_bytes().chunks(chunk_size).enumerate() {
        let more = if index + 1 < total_chunks { 1 } else { 0 };
        let payload = if index == 0 {
            // z=-1 removed to force overlay visibility (debug)
            format!(
                "\x1b_Gf=100,a=T,c={},r={},i={},m={};",
                area.width, area.height, image_id, more
            )
        } else {
            format!("\x1b_Gm={};", more)
        };

        if is_tmux {
            write!(stdout, "\x1bPtmux;\x1b")?;
            // Escape ESC
            write!(stdout, "{}", payload.replace("\x1b", "\x1b\x1b"))?;
        } else {
            write!(stdout, "{}", payload)?;
        }

        if is_tmux {
            for byte in chunk {
                write!(stdout, "{}", *byte as char)?;
            }
            write!(stdout, "\x1b\x1b\\")?;
            write!(stdout, "\x1b\\")?;
        } else {
            stdout.write_all(chunk)?;
            write!(stdout, "\x1b\\")?;
        }
    }
    Ok(())
}

fn write_iterm2_image<W: Write>(
    stdout: &mut W,
    image: &[u8],
    area: ratatui::layout::Rect,
) -> Result<()> {
    let encoded = STANDARD.encode(image);
    write!(
        stdout,
        "\x1b]1337;File=inline=1;width={};height={};preserveAspectRatio=1:",
        area.width, area.height
    )?;
    stdout.write_all(encoded.as_bytes())?;
    write!(stdout, "\x07")?;
    Ok(())
}
