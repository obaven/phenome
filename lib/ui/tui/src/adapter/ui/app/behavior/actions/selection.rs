use anyhow::Result;

use crate::app::App;

impl App {
    pub fn select_next_action(&mut self) {
        let actions = self.runtime.registry().actions();
        if actions.is_empty() {
            return;
        }
        let next = match self.action_state.selected() {
            Some(index) => (index + 1) % actions.len(),
            None => 0,
        };
        self.action_state.select(Some(next));
        self.sync_action_scroll(next);
    }

    pub fn select_previous_action(&mut self) {
        let actions = self.runtime.registry().actions();
        if actions.is_empty() {
            return;
        }
        let prev = match self.action_state.selected() {
            Some(index) => {
                if index == 0 {
                    actions.len() - 1
                } else {
                    index - 1
                }
            }
            None => 0,
        };
        self.action_state.select(Some(prev));
        self.sync_action_scroll(prev);
    }

    pub fn trigger_selected_action(&mut self) -> Result<()> {
        if let Some(selected) = self.action_state.selected() {
            let action = self.runtime.registry().actions().get(selected).cloned();
            if let Some(action) = action {
                self.request_action(
                    action.id,
                    action.label,
                    action.safety,
                    action.requires_confirmation,
                )?;
            }
        }
        Ok(())
    }
}
