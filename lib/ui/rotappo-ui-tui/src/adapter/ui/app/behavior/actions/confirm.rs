use anyhow::Result;

use phenome_domain::{ActionId, ActionSafety, Event, EventLevel};

use super::super::super::{App, ConfirmPrompt};

impl App {
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
                format!("Confirmation required: {label}"),
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
                    format!("Action canceled: {label}", label = confirm.label),
                ));
            }
        }
        Ok(())
    }
}
