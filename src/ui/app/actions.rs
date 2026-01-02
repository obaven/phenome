use anyhow::Result;

use crate::runtime::{ActionId, ActionSafety, Event, EventLevel};

use super::{App, ConfirmPrompt};

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
                self.mark_action_flash(selected);
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

    pub fn request_action(
        &mut self,
        action_id: ActionId,
        label: &str,
        safety: ActionSafety,
        requires_confirmation: bool,
    ) -> Result<()> {
        if requires_confirmation || safety == ActionSafety::Destructive {
            self.confirm = Some(ConfirmPrompt {
                action_id,
                label: label.to_string(),
                safety,
            });
            self.runtime.events_mut().push(Event::new(
                EventLevel::Warn,
                format!("Confirmation required: {}", label),
            ));
            return Ok(());
        }
        self.runtime.trigger_action(action_id)
    }

    pub fn confirm_action(&mut self, approved: bool) -> Result<()> {
        if let Some(confirm) = self.confirm.take() {
            if approved {
                self.runtime.trigger_action(confirm.action_id)?;
            } else {
                self.runtime.events_mut().push(Event::new(
                    EventLevel::Warn,
                    format!("Action canceled: {}", confirm.label),
                ));
            }
        }
        Ok(())
    }

    pub fn cycle_log_interval(&mut self) {
        let next = match self.ui.log_interval.as_secs() {
            1 => 2,
            2 => 5,
            5 => 10,
            _ => 1,
        };
        self.ui.log_interval = std::time::Duration::from_secs(next);
    }

    pub fn mark_action_flash(&mut self, index: usize) {
        self.ui.action_flash_index = Some(index);
        self.ui.action_flash_at = Some(std::time::Instant::now());
    }

    pub fn action_flash_active(&self, index: usize) -> bool {
        let Some(flash_at) = self.ui.action_flash_at else {
            return false;
        };
        if self.ui.action_flash_index != Some(index) {
            return false;
        }
        flash_at.elapsed() < std::time::Duration::from_millis(800)
    }

    pub fn filtered_events(&self) -> Vec<&crate::runtime::Event> {
        self.ui.log_cache.iter().collect()
    }

    pub fn refresh_log_cache(&mut self, force: bool) {
        if !force {
            if self.ui.log_paused {
                return;
            }
            if self.ui.last_log_emit.elapsed() < self.ui.log_interval {
                return;
            }
        }
        self.ui.last_log_emit = std::time::Instant::now();
        self.ui.log_cache = self
            .runtime
            .events()
            .iter()
            .filter(|event| self.ui.log_filter.matches(event.level))
            .cloned()
            .collect();
    }
}
