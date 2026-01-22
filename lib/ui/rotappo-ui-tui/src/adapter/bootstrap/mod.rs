use anyhow::Result;
use phenome_ports::PortSet;

pub mod panels;
mod ui;

pub use ui::{app, state, terminal, utils};

use self::ui::app::BootstrapApp;
use self::ui::terminal::run_app;
use crate::terminal::TerminalGuard;

pub fn start_bootstrap(ports: PortSet) -> Result<()> {
    let mut terminal_guard = TerminalGuard::new()?;
    let mut app = BootstrapApp::new(ports);
    run_app(terminal_guard.terminal_mut(), &mut app)
}
