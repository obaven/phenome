use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};

use super::BootstrapApp;

mod logs;
mod menu;

impl BootstrapApp {
    pub fn handle_key_event(&mut self, key: KeyEvent) -> Result<()> {
        if self.ui.show_summary {
            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => self.should_quit = true,
                KeyCode::Char('b') => self.ui.show_summary = false,
                _ => {}
            }
            return Ok(());
        }

        if self.ui.show_logs {
            return self.handle_logs_input(key);
        }

        if self.ui.menu_state.active {
            return self.handle_menu_input(key);
        }

        match key.code {
            KeyCode::Char('q') => self.should_quit = true,
            KeyCode::Char('m') => self.ui.menu_state.open(),
            KeyCode::Char('e') => self.toggle_expand_selected(),
            KeyCode::Char('c') => self.toggle_layer_collapse(),
            KeyCode::Tab => self.ui.focus = self.ui.focus.toggle(),
            KeyCode::Up => self.move_selection(-1),
            KeyCode::Down => self.move_selection(1),
            KeyCode::PageUp => self.scroll(-5),
            KeyCode::PageDown => self.scroll(5),
            _ => {}
        }
        Ok(())
    }
}
