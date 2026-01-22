use anyhow::Result;
use primer::application::events::InteractiveCommand;
use crossterm::event::{KeyCode, KeyEvent};
use std::time::Duration;

use super::super::BootstrapApp;

impl BootstrapApp {
    pub fn handle_menu_input(&mut self, key: KeyEvent) -> Result<()> {
        if self.ui.menu_state.confirming {
            match key.code {
                KeyCode::Char('y') | KeyCode::Char('Y') => {
                    if let Some(cmd) = self.ui.menu_state.pending_command.take() {
                        self.ports.bootstrap.send_command(cmd)?;
                    }
                    self.ui.menu_state.clear();
                }
                KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                    self.ui.menu_state.clear();
                }
                _ => {}
            }
            return Ok(());
        }

        if self.ui.menu_state.timeout_input.is_some() {
            return self.handle_timeout_input(key);
        }

        match key.code {
            KeyCode::Up => {
                self.ui.menu_state.selected = self.ui.menu_state.selected.saturating_sub(1)
            }
            KeyCode::Down => {
                self.ui.menu_state.selected = self.ui.menu_state.selected.saturating_add(1)
            }
            KeyCode::Enter => self.execute_menu_action()?,
            KeyCode::Esc => self.ui.menu_state.clear(),
            _ => {}
        }
        Ok(())
    }

    fn handle_timeout_input(&mut self, key: KeyEvent) -> Result<()> {
        let Some(input) = self.ui.menu_state.timeout_input.as_mut() else {
            return Ok(());
        };

        match key.code {
            KeyCode::Enter => {
                let value = input.parse::<u64>().unwrap_or(0);
                if let Some(component) = self.selected_component_id() {
                    let cmd = InteractiveCommand::AdjustTimeout {
                        id: component,
                        new_timeout: Duration::from_secs(value),
                    };
                    self.ports.bootstrap.send_command(cmd)?;
                }
                self.ui.menu_state.clear();
            }
            KeyCode::Char(ch) if ch.is_ascii_digit() => input.push(ch),
            KeyCode::Backspace => {
                input.pop();
            }
            KeyCode::Esc => self.ui.menu_state.clear(),
            _ => {}
        }
        Ok(())
    }
}
