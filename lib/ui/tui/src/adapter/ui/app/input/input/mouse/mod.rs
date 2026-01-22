use anyhow::Result;
use crossterm::event::{MouseButton, MouseEvent, MouseEventKind};

use crate::app::{App, NavView};

mod drag;
mod graph;
mod navbar;

impl App {
    pub fn handle_mouse_event(&mut self, mouse: MouseEvent) -> Result<()> {
        if self.confirm.is_some() {
            return Ok(());
        }
        let pos = (mouse.column, mouse.row);
        match mouse.kind {
            MouseEventKind::Down(MouseButton::Left) => {
                self.ui.mouse_pos = Some(pos);
                if navbar::handle_navbar_click(self, mouse.column, mouse.row) {
                    return Ok(());
                }
                if graph::handle_graph_click(self, mouse.column, mouse.row) {
                    return Ok(());
                }
            }
            MouseEventKind::Down(MouseButton::Right) => {
                self.ui.mouse_pos = Some(pos);
            }
            MouseEventKind::ScrollDown => {
                self.ui.mouse_pos = Some(pos);
                let view = self.active_view();
                let is_detail_hover = self.ui.show_detail_panel
                    && self.ui.detail_area.contains((mouse.column, mouse.row).into());

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
                self.ui.mouse_pos = Some(pos);
                let view = self.active_view();
                let is_detail_hover = self.ui.show_detail_panel
                    && self.ui.detail_area.contains((mouse.column, mouse.row).into());

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
                drag::handle_graph_drag(self, mouse.column, mouse.row);
                self.ui.mouse_pos = Some(pos);
            }
            MouseEventKind::Moved => self.update_hover(mouse.column, mouse.row),
            _ => {}
        }
        Ok(())
    }
}
