pub mod app;
pub mod panels;
pub mod state;
pub mod util;

use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event as CrosstermEvent},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    prelude::Frame,
    widgets::Clear,
    Terminal,
};
use std::io::{self, Stdout};
use std::time::Duration;

use crate::adapters::bootstrappo::BootstrappoBackend;
use crate::ui::app::App;

pub fn start() -> Result<()> {
    let backend = BootstrappoBackend::from_env()?;
    let mut terminal_guard = TerminalGuard::new()?;
    let mut app = App::new(backend);
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

fn render(frame: &mut Frame, app: &mut App) {
    let size = frame.area();
    app.ui.screen_area = size;
    let help_height = if app.ui.collapsed_help {
        2
    } else {
        size.height.saturating_mul(30).saturating_div(100).max(6)
    };
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0), Constraint::Length(help_height)])
        .split(size);
    app.ui.body_area = layout[1];

    panels::render_header(frame, layout[0], app);

    let body = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(30),
            Constraint::Percentage(35),
            Constraint::Percentage(35),
        ])
        .split(layout[1]);

    let collapsed = 2u16;
    let plan_progress_height = if app.ui.collapsed_plan_progress { collapsed } else { 3 };
    let snapshot_height = if app.ui.collapsed_snapshot { collapsed } else { 4 };
    let left_top_height = plan_progress_height.saturating_add(snapshot_height);
    let capabilities_constraint = if app.ui.collapsed_capabilities {
        Constraint::Length(collapsed)
    } else {
        Constraint::Min(0)
    };
    let left = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(left_top_height), capabilities_constraint])
        .split(body[0]);

    panels::render_plan(frame, left[0], app);
    panels::render_capabilities(frame, left[1], app);
    let middle_panel = app.middle_aux_panel();
    if app.ui.collapsed_plan_steps {
        let middle = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0), Constraint::Length(collapsed)])
            .split(body[1]);
        match middle_panel {
            Some(crate::ui::app::PanelId::Logs) => panels::render_logs(frame, middle[0], app),
            Some(crate::ui::app::PanelId::Help) => panels::render_footer(frame, middle[0], app),
            _ => frame.render_widget(Clear, middle[0]),
        }
        panels::render_plan_steps(frame, middle[1], app);
    } else {
        panels::render_plan_steps(frame, body[1], app);
    }

    let right = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(45), Constraint::Percentage(55)])
        .split(body[2]);
    let actions_constraint = if app.ui.collapsed_actions {
        Constraint::Length(collapsed)
    } else {
        Constraint::Min(8)
    };
    let problems_constraint = if app.ui.collapsed_problems {
        Constraint::Length(collapsed)
    } else {
        Constraint::Min(4)
    };
    let log_controls_constraint = Constraint::Length(app.log_controls_height());
    let logs_constraint = if app.ui.collapsed_logs {
        Constraint::Length(collapsed)
    } else {
        Constraint::Min(6)
    };
    let right_left = Layout::default()
        .direction(Direction::Vertical)
        .constraints([actions_constraint, problems_constraint])
        .split(right[0]);
    let right_right = Layout::default()
        .direction(Direction::Vertical)
        .constraints([log_controls_constraint, logs_constraint])
        .split(right[1]);

    panels::render_actions(frame, right_left[0], app);
    panels::render_problems(frame, right_left[1], app);
    panels::render_log_controls(frame, right_right[0], app);
    if middle_panel != Some(crate::ui::app::PanelId::Logs) {
        panels::render_logs(frame, right_right[1], app);
    }
    app.ui.help_area = ratatui::layout::Rect::default();
    app.ui.settings_area = ratatui::layout::Rect::default();
    app.ui.settings_controls_row = None;
    if middle_panel == Some(crate::ui::app::PanelId::Help) {
        if !app.ui.collapsed_settings {
            panels::render_settings(frame, layout[2], app);
        } else if !app.ui.collapsed_help {
            panels::render_footer(frame, layout[2], app);
        }
    } else if !app.ui.collapsed_help && !app.ui.collapsed_settings {
        let footer = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(35), Constraint::Percentage(65)])
            .split(layout[2]);
        panels::render_footer(frame, footer[0], app);
        panels::render_settings(frame, footer[1], app);
    } else if !app.ui.collapsed_help {
        panels::render_footer(frame, layout[2], app);
    } else if !app.ui.collapsed_settings {
        panels::render_settings(frame, layout[2], app);
    } else {
        panels::render_footer(frame, layout[2], app);
    }

    panels::render_confirmation(frame, app);
    panels::render_tooltip(frame, app);
}
