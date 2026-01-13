use anyhow::{Context, Result};
use async_trait::async_trait;
use chrono::Utc;
use std::sync::Arc;
use tokio::time::{Duration, interval};

use rotappo_domain::{ScheduleId, ScheduleStatus, ScheduledAction};
use rotappo_ports::SchedulerPort;

use crate::storage::StoragePort;

#[derive(Clone)]
pub struct SchedulerService {
    storage: Arc<dyn StoragePort>,
}

impl SchedulerService {
    pub fn new(storage: Arc<dyn StoragePort>) -> Self {
        Self { storage }
    }

    pub async fn run_scheduler_loop(&self, kube_client: kube::Client) {
        let mut interval_timer = interval(Duration::from_secs(60));
        loop {
            interval_timer.tick().await;
            if let Err(e) = self.check_and_execute(&kube_client).await {
                tracing::error!("Scheduler loop error: {}", e);
            }
        }
    }

    async fn check_and_execute(&self, _kube_client: &kube::Client) -> Result<()> {
        let all = self.storage.get_all_schedules().await?;
        let now = Utc::now().timestamp_millis();

        for mut action in all {
            // Check if due and pending
            if action.execute_at <= now && matches!(action.status, ScheduleStatus::Pending) {
                // Execute
                tracing::info!("Executing scheduled action: {}", action.id);
                // Mark as Executing
                action.status = ScheduleStatus::Executing;
                self.storage.update_schedule(action.clone()).await?;

                // TODO: Execute via kube-rs (not fully implemented in this ticket scope/snippet, but stubbed)
                // Implement execution logic based on action type (ScaleDeployment, etc.)
                let success = true;

                if success {
                    action.status = ScheduleStatus::Completed;
                } else {
                    action.status = ScheduleStatus::Failed {
                        error: "Execution stub".to_string(),
                    };
                }
                self.storage.update_schedule(action).await?;
            }
        }
        Ok(())
    }
}

#[async_trait]
impl SchedulerPort for SchedulerService {
    async fn schedule_action(&self, action: ScheduledAction) -> Result<ScheduleId> {
        if action.id.is_empty() {
            anyhow::bail!("scheduled action id is required");
        }
        self.storage.insert_schedule(action.clone()).await?;
        Ok(action.id)
    }

    async fn cancel_schedule(&self, id: ScheduleId) -> Result<()> {
        // Need to fetch, modify, update
        let all = self.storage.get_all_schedules().await?;
        if let Some(mut action) = all.into_iter().find(|a| a.id == id) {
            action.status = ScheduleStatus::Cancelled;
            self.storage.update_schedule(action).await?;
        }
        Ok(())
    }

    async fn list_scheduled(&self) -> Result<Vec<ScheduledAction>> {
        self.storage.get_all_schedules().await
    }
}
