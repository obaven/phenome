use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::app::{App, GraphDirection, NavView};

impl App {
    pub fn handle_graph_key(&mut self, key: KeyEvent) -> Result<bool> {
        let view = self.active_view();
        if !matches!(view, NavView::TopologyDagGraph | NavView::TopologyDualGraph) {
            return Ok(false);
        }
        match key.code {
            KeyCode::Enter => {
                if let Some(_id) = self.graph.selected_id() {
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
}
