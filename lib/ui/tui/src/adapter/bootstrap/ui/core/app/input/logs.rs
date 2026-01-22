use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};

use super::super::BootstrapApp;

impl BootstrapApp {
    pub fn handle_logs_input(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => {
                self.ui.show_logs = false;
            }
            KeyCode::Up => self.scroll_logs(1),
            KeyCode::Down => self.scroll_logs(-1),
            KeyCode::PageUp => self.scroll_logs(5),
            KeyCode::PageDown => self.scroll_logs(-5),
            KeyCode::End => self.ui.log_scroll = 0,
            _ => {}
        }
        Ok(())
    }
}
