//! Keyboard-driven event handling for the TUI.

use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use super::{App, GraphDirection, NavView};

impl App {
    /// Handle a keyboard event from crossterm.
    pub fn handle_key_event(&mut self, key: KeyEvent) -> Result<()> {
        if self.ui.search_active {
            match key.code {
                KeyCode::Esc => {
                    self.ui.search_active = false;
                    self.ui.search_query.clear();
                }
                KeyCode::Enter => {
                    self.execute_search();
                    self.ui.search_active = false;
                    self.ui.search_query.clear();
                }
                KeyCode::Backspace => {
                    self.ui.search_query.pop();
                }
                KeyCode::Char(c) => {
                    self.ui.search_query.push(c);
                }
                _ => {}
            }
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

    fn handle_graph_key(&mut self, key: KeyEvent) -> Result<bool> {
        let view = self.active_view();
        if !matches!(view, NavView::TopologyDagGraph | NavView::TopologyDualGraph) {
            return Ok(false);
        }
        match key.code {
            KeyCode::Enter => {
                if let Some(_id) = self.graph.selected_id() {
                    // Toggle detail panel
                    self.ui.show_detail_panel = !self.ui.show_detail_panel;
                } else {
                    self.activate_graph_selection();
                }
                Ok(true)
            }
            KeyCode::Char('/') => {
                self.ui.search_active = true;
                Ok(true)
            }
            KeyCode::Char('+') | KeyCode::Char('=') => {
                self.graph.zoom_in();
                Ok(true)
            }
            KeyCode::Char('-') => {
                self.graph.zoom_out();
                Ok(true)
            }
            KeyCode::Char('0') => {
                self.graph.reset_view();
                Ok(true)
            }
            KeyCode::Up if key.modifiers.contains(KeyModifiers::SHIFT) => {
                self.ui.detail_scroll = self.ui.detail_scroll.saturating_sub(1);
                Ok(true)
            }
            KeyCode::Down if key.modifiers.contains(KeyModifiers::SHIFT) => {
                self.ui.detail_scroll = self.ui.detail_scroll.saturating_add(1);
                Ok(true)
            }
            KeyCode::Left | KeyCode::Char('a') => {
                if key.modifiers.contains(KeyModifiers::SHIFT) || key.code == KeyCode::Char('a') {
                    self.pan_graph(GraphDirection::Left);
                } else {
                    self.graph.select_direction(GraphDirection::Left);
                }
                Ok(true)
            }
            KeyCode::Right | KeyCode::Char('d') => {
                if key.modifiers.contains(KeyModifiers::SHIFT) || key.code == KeyCode::Char('d') {
                    self.pan_graph(GraphDirection::Right);
                } else {
                    self.graph.select_direction(GraphDirection::Right);
                }
                Ok(true)
            }
            KeyCode::Up | KeyCode::Char('w') => {
                if key.modifiers.contains(KeyModifiers::SHIFT) || key.code == KeyCode::Char('w') {
                    self.pan_graph(GraphDirection::Up);
                } else {
                    self.graph.select_direction(GraphDirection::Up);
                }
                Ok(true)
            }
            KeyCode::Down | KeyCode::Char('s') => {
                if key.modifiers.contains(KeyModifiers::SHIFT) || key.code == KeyCode::Char('s') {
                    self.pan_graph(GraphDirection::Down);
                } else {
                    self.graph.select_direction(GraphDirection::Down);
                }
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn pan_graph(&mut self, direction: GraphDirection) {
        let Some(layout) = self.graph.layout() else {
            return;
        };
        let (step_x, step_y) = self.graph.pan_step(layout, self.ui.assembly_area);
        match direction {
            GraphDirection::Left => self.graph.pan(-step_x, 0.0),
            GraphDirection::Right => self.graph.pan(step_x, 0.0),
            GraphDirection::Up => self.graph.pan(0.0, step_y),
            GraphDirection::Down => self.graph.pan(0.0, -step_y),
        }
    }

    fn execute_search(&mut self) {
        let query = self.ui.search_query.to_lowercase();
        if query.is_empty() {
            return;
        }
        let Some(layout) = self.graph.layout() else {
            return;
        };

        // Find best match: exact id match, then substring in label
        let best = layout
            .nodes
            .iter()
            .find(|n| n.id.to_lowercase() == query)
            .or_else(|| {
                layout
                    .nodes
                    .iter()
                    .find(|n| n.label.to_lowercase().contains(&query))
            })
            .map(|n| n.id.clone());

        if let Some(id) = best {
            self.graph.select_node(&id);
        }
    }
}
