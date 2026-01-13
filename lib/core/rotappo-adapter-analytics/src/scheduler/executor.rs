use anyhow::Result;

use rotappo_domain::ScheduledAction;

pub async fn execute_action(_action: &ScheduledAction) -> Result<()> {
    Ok(())
}
