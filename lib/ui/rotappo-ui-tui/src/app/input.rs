use anyhow::Result;
use crossterm::event::{
    KeyCode, KeyEvent, KeyEventKind, MouseButton, MouseEvent, MouseEventKind,
};
use ratatui::layout::Margin;
use std::time::Duration;

use rotappo_ui_presentation::logging::{LogFilter, LOG_INTERVALS_SECS};
use rotappo_domain::{Event, EventLevel};

use super::App;
use crate::state::HoldState;

impl App {
    pub fn handle_confirm_key(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Char('y') | KeyCode::Enter => self.confirm_action(true)?,
            KeyCode::Char('n') | KeyCode::Esc => self.confirm_action(false)?,
            _ => {}
        }
        Ok(())
    }

    pub fn handle_mouse_event(&mut self, mouse: MouseEvent) -> Result<()> {
        if self.confirm.is_some() {
            return Ok(());
        }
        self.ui.mouse_pos = Some((mouse.column, mouse.row));
        match mouse.kind {
            MouseEventKind::Down(MouseButton::Left) => {
                let click_pos = (mouse.column, mouse.row).into();
                if self.handle_header_click(mouse.column, mouse.row) {
                    return Ok(());
                }
                if self.ui.log_menu_pinned
                    && !self.ui.log_menu_area.contains(click_pos)
                    && !self.log_menu_trigger_contains(click_pos)
                {
                    self.close_log_menu();
                }
                if self.handle_log_menu_click(mouse.column, mouse.row) {
                    return Ok(());
                }
                if self.handle_log_tag_click(mouse.column, mouse.row) {
                    return Ok(());
                }
                if self.handle_settings_click(mouse.column, mouse.row) {
                    return Ok(());
                }
                self.handle_action_click(mouse.column, mouse.row, false)?;
            }
            MouseEventKind::Down(MouseButton::Right) => {
                self.handle_action_click(mouse.column, mouse.row, true)?;
            }
            MouseEventKind::ScrollDown => {
                self.update_hover(mouse.column, mouse.row);
                self.scroll_active_panel(1);
            }
            MouseEventKind::ScrollUp => {
                self.update_hover(mouse.column, mouse.row);
                self.scroll_active_panel(-1);
            }
            MouseEventKind::Moved => self.update_hover(mouse.column, mouse.row),
            _ => {}
        }
        Ok(())
    }

    fn handle_log_menu_click(&mut self, column: u16, row: u16) -> bool {
        if self.ui.log_menu_len == 0 {
            return false;
        }
        if !self.ui.log_menu_area.contains((column, row).into()) {
            return false;
        }
        let inner = self.ui.log_menu_area.inner(Margin {
            horizontal: 1,
            vertical: 1,
        });
        if !inner.contains((column, row).into()) {
            return false;
        }
        let index = row.saturating_sub(inner.y) as usize;
        if index >= self.ui.log_menu_len {
            return false;
        }
        if let Some(mode) = self.ui.log_menu_mode {
            self.apply_log_menu_action(mode, index);
        }
        true
    }

    fn apply_log_menu_action(&mut self, mode: crate::state::LogMenuMode, index: usize) {
        let mut refresh = matches!(mode, crate::state::LogMenuMode::Filter);
        match mode {
            crate::state::LogMenuMode::Filter => match index {
                0 => self.ui.log_config.filter = LogFilter::All,
                1 => self.ui.log_config.filter = LogFilter::Info,
                2 => self.ui.log_config.filter = LogFilter::Warn,
                3 => self.ui.log_config.filter = LogFilter::Error,
                _ => {}
            },
            crate::state::LogMenuMode::Stream => {
                if let Some(&secs) = LOG_INTERVALS_SECS.get(index) {
                    self.ui.log_config.interval = Duration::from_secs(secs);
                    refresh = true;
                } else if index == LOG_INTERVALS_SECS.len() {
                    self.ui.log_paused = !self.ui.log_paused;
                    refresh = true;
                }
            }
        }
        if refresh {
            self.refresh_log_cache(true);
        }
        self.close_log_menu();
    }

    fn handle_log_tag_click(&mut self, column: u16, row: u16) -> bool {
        let pos = (column, row).into();
        if self.ui.log_filter_tag_area.contains(pos) {
            self.toggle_log_menu(crate::state::LogMenuMode::Filter);
            return true;
        }
        if self.ui.log_stream_tag_area.contains(pos) {
            self.toggle_log_menu(crate::state::LogMenuMode::Stream);
            return true;
        }
        if self.ui.collapsed_log_controls {
            return false;
        }
        let inner = self.ui.log_controls_area.inner(Margin {
            horizontal: 1,
            vertical: 1,
        });
        if inner.height == 0
            || row != inner.y
            || column < inner.x
            || column >= inner.x.saturating_add(inner.width)
        {
            return false;
        }
        let filter_start =
            self.ui.log_filter_tag_area.x.saturating_sub(super::FILTER_LABEL.len() as u16);
        let filter_end = self
            .ui
            .log_filter_tag_area
            .x
            .saturating_add(self.ui.log_filter_tag_area.width);
        if column >= filter_start && column < filter_end {
            self.toggle_log_menu(crate::state::LogMenuMode::Filter);
            return true;
        }
        let stream_start =
            self.ui.log_stream_tag_area.x.saturating_sub(super::STREAM_LABEL.len() as u16);
        let stream_end = self
            .ui
            .log_stream_tag_area
            .x
            .saturating_add(self.ui.log_stream_tag_area.width);
        if column >= stream_start && column < stream_end {
            self.toggle_log_menu(crate::state::LogMenuMode::Stream);
            return true;
        }
        false
    }

    fn toggle_log_menu(&mut self, mode: crate::state::LogMenuMode) {
        if self.ui.log_menu_pinned && self.ui.log_menu_mode == Some(mode) {
            self.close_log_menu();
        } else {
            self.ui.log_menu_mode = Some(mode);
            self.ui.log_menu_pinned = true;
        }
    }

    fn handle_settings_click(&mut self, column: u16, row: u16) -> bool {
        if self.ui.collapsed_settings {
            return false;
        }
        let Some(controls_row) = self.ui.settings_controls_row else {
            return false;
        };
        if row != controls_row {
            return false;
        }
        let inner = self.ui.settings_area.inner(Margin {
            horizontal: 1,
            vertical: 1,
        });
        if !inner.contains((column, row).into()) {
            return false;
        }
        let apply_start = inner.x;
        let apply_end = apply_start.saturating_add(7);
        let cancel_start = apply_start.saturating_add(9);
        let cancel_end = cancel_start.saturating_add(8);
        if column >= apply_start && column < apply_end {
            self.ui.settings_selected = 0;
            self.runtime.events_mut().push(Event::new(
                EventLevel::Info,
                "Settings: apply (stub)",
            ));
            return true;
        }
        if column >= cancel_start && column < cancel_end {
            self.ui.settings_selected = 1;
            self.runtime.events_mut().push(Event::new(
                EventLevel::Info,
                "Settings: cancel (stub)",
            ));
            return true;
        }
        false
    }

    pub fn handle_action_click(&mut self, column: u16, row: u16, trigger: bool) -> Result<()> {
        if self.ui.collapsed_actions {
            return Ok(());
        }
        if !self.ui.actions_area.contains((column, row).into()) {
            return Ok(());
        }

        let inner = self.ui.actions_area.inner(Margin {
            horizontal: 1,
            vertical: 1,
        });
        if !inner.contains((column, row).into()) {
            return Ok(());
        }

        let actions = self.runtime.registry().actions();
        if actions.is_empty() {
            return Ok(());
        }

        let row_offset = row.saturating_sub(inner.y) as usize;
        let item_height = 2usize;
        let index = row_offset / item_height + self.ui.actions_scroll as usize;
        if index >= actions.len() {
            return Ok(());
        }

        self.action_state.select(Some(index));
        self.sync_action_scroll(index);
        self.runtime.events_mut().push(Event::new(
            EventLevel::Info,
            format!("Mouse select: action {} at ({},{})", index + 1, column, row),
        ));
        if trigger {
            self.mark_action_flash(index);
            self.trigger_selected_action()?;
        }
        Ok(())
    }

    pub fn handle_hold_key(&mut self, key: &KeyEvent) -> bool {
        let pressed = key.kind == KeyEventKind::Press;
        let released = key.kind == KeyEventKind::Release;
        let KeyCode::Char(ch) = key.code else {
            return false;
        };
        if !matches!(ch, 'p' | 'u') {
            return false;
        }
        if pressed {
            self.start_hold(ch);
            return true;
        }
        if released {
            self.finish_hold(ch);
            return true;
        }
        false
    }

    pub fn start_hold(&mut self, key: char) {
        self.ui.hold_state = Some(HoldState {
            key,
            started_at: std::time::Instant::now(),
            triggered: false,
        });
    }

    pub fn finish_hold(&mut self, key: char) {
        if let Some(hold) = &self.ui.hold_state {
            if hold.key == key {
                if !hold.triggered && key == 'p' {
                    self.ui.log_paused = !self.ui.log_paused;
                }
                self.ui.hold_state = None;
            }
        }
    }
}
