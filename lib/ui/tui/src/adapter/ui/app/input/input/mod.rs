use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};

use crate::app::App;

mod hold;
mod mouse;

impl App {
    pub fn handle_confirm_key(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Char('y') | KeyCode::Enter => self.confirm_action(true)?,
            KeyCode::Char('n') | KeyCode::Esc => self.confirm_action(false)?,
            _ => {}
        }
        Ok(())
    }
}
