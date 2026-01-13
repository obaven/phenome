use anyhow::Result;
use async_trait::async_trait;

use rotappo_domain::{ScheduleId, ScheduledAction};

#[async_trait]
pub trait SchedulerPort: Send + Sync {
    async fn schedule_action(&self, action: ScheduledAction) -> Result<ScheduleId>;
    async fn cancel_schedule(&self, id: ScheduleId) -> Result<()>;
    async fn list_scheduled(&self) -> Result<Vec<ScheduledAction>>;
}
