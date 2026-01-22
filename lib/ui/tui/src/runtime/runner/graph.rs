use anyhow::Result;
use crossterm::{cursor::MoveTo, queue};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use std::io::Write;
use std::io::Stdout;

use crate::app::{App, PanelId, TerminalImageProtocol};

use super::iterm::write_iterm2_image;
use super::kitty::write_kitty_image;

pub(super) fn render_graph(
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
