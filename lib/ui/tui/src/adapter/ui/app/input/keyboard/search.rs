use crate::app::App;
use crossterm::event::{KeyCode, KeyEvent};

impl App {
    pub fn handle_search_key(&mut self, key: KeyEvent) -> bool {
        if !self.ui.search_active {
            return false;
        }
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
        true
    }

    pub fn execute_search(&mut self) {
        let query = self.ui.search_query.to_lowercase();
        if query.is_empty() {
            return;
        }
        let Some(layout) = self.graph.layout() else {
            return;
        };

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
