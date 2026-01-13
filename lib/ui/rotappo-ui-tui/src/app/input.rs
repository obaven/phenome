use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, MouseButton, MouseEvent, MouseEventKind};
use ratatui::layout::Margin;
use std::time::Duration;

use rotappo_domain::{Event, EventLevel};
use rotappo_ui_presentation::logging::{LOG_INTERVALS_SECS, LogFilter};

use super::{App, NavView};
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
                if self.handle_navbar_click(mouse.column, mouse.row) {
                    return Ok(());
                }
                if self.handle_header_click(mouse.column, mouse.row) {
                    return Ok(());
                }
                if self.handle_graph_click(mouse.column, mouse.row) {
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
                let view = self.active_view();
                let is_detail_hover = self.ui.show_detail_panel && self.ui.detail_area.contains((mouse.column, mouse.row).into());
                
                if is_detail_hover {
                     self.ui.detail_scroll = self.ui.detail_scroll.saturating_add(1);
                } else if matches!(view, NavView::TopologyDagGraph | NavView::TopologyDualGraph) {
                    self.graph.zoom_out();
                } else {
                    self.update_hover(mouse.column, mouse.row);
                    self.scroll_active_panel(1);
                }
            }
            MouseEventKind::ScrollUp => {
                let view = self.active_view();
                let is_detail_hover = self.ui.show_detail_panel && self.ui.detail_area.contains((mouse.column, mouse.row).into());

                if is_detail_hover {
                    self.ui.detail_scroll = self.ui.detail_scroll.saturating_sub(1);
                } else if matches!(view, NavView::TopologyDagGraph | NavView::TopologyDualGraph) {
                    self.graph.zoom_in();
                } else {
                    self.update_hover(mouse.column, mouse.row);
                    self.scroll_active_panel(-1);
                }
            }
            MouseEventKind::Drag(MouseButton::Left) => {
                let view = self.active_view();
                if matches!(view, NavView::TopologyDagGraph | NavView::TopologyDualGraph) {
                    // Calculate delta (we need previous mouse pos, which updates in 'Moved' or store 'drag_start'?)
                    // `crossterm` Drag events just give current pos.
                    // We need self.ui.mouse_pos to compare?
                    if let Some((prev_c, prev_r)) = self.ui.mouse_pos {
                        let dx = (mouse.column as i16 - prev_c as i16) as f64;
                        let dy = (mouse.row as i16 - prev_r as i16) as f64;

                        if let Some(layout) = self.graph.layout() {
                            let bounds = self.graph.view_bounds_for(layout, self.ui.assembly_area);
                            let screen_w = self.ui.assembly_area.width.max(1) as f64;
                            let screen_h = self.ui.assembly_area.height.max(1) as f64;

                            let graph_dx = dx * (bounds.x_max - bounds.x_min) / screen_w;
                            let graph_dy = dy * (bounds.y_max - bounds.y_min) / screen_h;

                            // Invert for "drag paper" feel
                            self.graph.pan(-graph_dx, graph_dy);
                        }
                    }
                }
                self.ui.mouse_pos = Some((mouse.column, mouse.row));
            }
            MouseEventKind::Moved => self.update_hover(mouse.column, mouse.row),
            _ => {}
        }
        Ok(())
    }

    fn handle_navbar_click(&mut self, column: u16, row: u16) -> bool {
        let pos = (column, row).into();
        if self.ui.nav_flyout_area.contains(pos) {
            for (index, area) in self
                .ui
                .nav_flyout_item_areas
                .iter()
                .take(self.ui.nav_flyout_count)
                .enumerate()
            {
                if area.contains(pos) {
                    self.activate_nav_sub(index);
                    return true;
                }
            }
        }
        for (index, area) in self.ui.navbar_item_areas.iter().enumerate() {
            if area.contains(pos) {
                let nav = crate::app::NavSection::from_index(index);
                self.set_active_nav(nav);
                return true;
            }
        }
        false
    }

    fn handle_graph_click(&mut self, column: u16, row: u16) -> bool {
        let view = self.active_view();
        if !matches!(view, NavView::TopologyDagGraph | NavView::TopologyDualGraph) {
            return false;
        }
        let area = self.ui.assembly_area;
        if area.width == 0 || area.height == 0 {
            return false;
        }
        if !area.contains((column, row).into()) {
            return false;
        }
        let Some(bounds) = self.graph.view_bounds(self.ui.assembly_area) else {
            return true;
        };
        let width = area.width.saturating_sub(1).max(1);
        let height = area.height.saturating_sub(1).max(1);
        let x_ratio = (column.saturating_sub(area.x) as f64) / (width as f64);
        let y_ratio = (row.saturating_sub(area.y) as f64) / (height as f64);
        let x = bounds.x_min + x_ratio * (bounds.x_max - bounds.x_min);
        let y = bounds.y_max - y_ratio * (bounds.y_max - bounds.y_min);
        if self.graph.select_node_at(x, y) {
            self.ui.show_detail_panel = true;
        }
        true
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
        let filter_start = self
            .ui
            .log_filter_tag_area
            .x
            .saturating_sub(super::FILTER_LABEL.len() as u16);
        let filter_end = self
            .ui
            .log_filter_tag_area
            .x
            .saturating_add(self.ui.log_filter_tag_area.width);
        if column >= filter_start && column < filter_end {
            self.toggle_log_menu(crate::state::LogMenuMode::Filter);
            return true;
        }
        let stream_start = self
            .ui
            .log_stream_tag_area
            .x
            .saturating_sub(super::STREAM_LABEL.len() as u16);
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
            self.runtime
                .events_mut()
                .push(Event::new(EventLevel::Info, "Settings: apply (stub)"));
            return true;
        }
        if column >= cancel_start && column < cancel_end {
            self.ui.settings_selected = 1;
            self.runtime
                .events_mut()
                .push(Event::new(EventLevel::Info, "Settings: cancel (stub)"));
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
            format!(
                "Mouse select: action {action} at ({column},{row})",
                action = index + 1
            ),
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
