use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::app::{App, NavView};

impl App {
    pub fn handle_key_event(&mut self, key: KeyEvent) -> Result<()> {
        if self.handle_search_key(key) {
            return Ok(());
        }

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
        if self.handle_graph_key(key)? {
            return Ok(());
        }

        let view = self.active_view();
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => self.should_quit = true,
            KeyCode::Char('r') => self.runtime.refresh_snapshot(),
            KeyCode::Char('f') => {
                if matches!(
                    view,
                    NavView::TerminalLogs | NavView::TerminalEvents | NavView::TerminalDiagnostics
                ) {
                    self.ui.log_config.filter = self.ui.log_config.filter.next();
                    self.refresh_log_cache(true);
                }
            }
            KeyCode::Char('n') => self.toggle_notifications_panel(),
            KeyCode::Char('w') => self.ui.auto_refresh = !self.ui.auto_refresh,
            KeyCode::Char('a') => self.set_active_nav(crate::app::NavSection::Analytics),
            KeyCode::Char('1') if self.active_nav() == crate::app::NavSection::Analytics => {
                self.set_nav_sub_index(0);
            }
            KeyCode::Char('2') if self.active_nav() == crate::app::NavSection::Analytics => {
                self.set_nav_sub_index(1);
            }
            KeyCode::Char('3') if self.active_nav() == crate::app::NavSection::Analytics => {
                self.set_nav_sub_index(2);
            }
            KeyCode::Char('4') if self.active_nav() == crate::app::NavSection::Analytics => {
                self.set_nav_sub_index(3);
            }
            KeyCode::Char('1') => self.set_active_nav(crate::app::NavSection::Analytics),
            KeyCode::Char('2') => self.set_active_nav(crate::app::NavSection::Topology),
            KeyCode::Char('3') => self.set_active_nav(crate::app::NavSection::Terminal),
            KeyCode::Left | KeyCode::BackTab => self.prev_nav(),
            KeyCode::Right | KeyCode::Tab => self.next_nav(),
            KeyCode::Char('[') => self.prev_nav_sub(),
            KeyCode::Char(']') => self.next_nav_sub(),
            KeyCode::Up | KeyCode::Char('k') => {
                if matches!(view, NavView::TerminalCommands) {
                    self.select_previous_action();
                } else {
                    self.prev_nav_sub();
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if matches!(view, NavView::TerminalCommands) {
                    self.select_next_action();
                } else {
                    self.next_nav_sub();
                }
            }
            KeyCode::Enter => {
                if matches!(view, NavView::TerminalCommands) {
                    self.trigger_selected_action()?;
                } else if let Some(item) = self.active_subitem() {
                    if item.action != crate::app::NavAction::None {
                        let index = self.nav_sub_index(self.active_nav());
                        self.activate_nav_sub(index);
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }
}
