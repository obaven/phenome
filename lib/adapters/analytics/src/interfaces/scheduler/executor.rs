use anyhow::Result;

use phenome_domain::ScheduledAction;

pub async fn execute_action(_action: &ScheduledAction) -> Result<()> {
    Ok(())
}
