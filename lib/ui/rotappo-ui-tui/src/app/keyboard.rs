//! Keyboard-driven event handling for the TUI.

use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use rotappo_domain::{Event, EventLevel};

use super::{App, PanelId};

impl App {
    /// Handle a keyboard event from crossterm.
    pub fn handle_key_event(&mut self, key: KeyEvent) -> Result<()> {
        if self.confirm.is_some() {
            return self.handle_confirm_key(key);
        }
        if self.handle_hold_key(&key) {
            return Ok(());
        }
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            self.should_quit = true;
            return Ok(());
        }

        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => self.should_quit = true,
            KeyCode::Char('r') => self.runtime.refresh_snapshot(),
            KeyCode::Char('f') => {
                self.ui.log_config.filter = self.ui.log_config.filter.next();
                self.refresh_log_cache(true);
            }
            KeyCode::Char('s') => self.toggle_settings_panel(),
            KeyCode::Char('w') => self.ui.auto_refresh = !self.ui.auto_refresh,
            KeyCode::Char('a') => self.handle_settings_shortcut(true),
            KeyCode::Char('c') => self.handle_settings_shortcut(false),
            KeyCode::Up | KeyCode::Char('k') => self.select_previous_action(),
            KeyCode::Down | KeyCode::Char('j') => self.select_next_action(),
            KeyCode::Enter => self.trigger_selected_action()?,
            _ => {}
        }
        Ok(())
    }

    fn handle_settings_shortcut(&mut self, apply: bool) {
        if self.panel_collapsed(PanelId::Settings) {
            return;
        }
        self.ui.settings_selected = if apply { 0 } else { 1 };
        let message = if apply {
            "Settings: apply (shortcut)"
        } else {
            "Settings: cancel (shortcut)"
        };
        self.runtime
            .events_mut()
            .push(Event::new(EventLevel::Info, message));
    }
}
