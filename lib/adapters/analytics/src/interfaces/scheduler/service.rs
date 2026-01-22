use anyhow::Result;
use async_trait::async_trait;
use chrono::Utc;
use std::sync::Arc;
use tokio::sync::watch;
use tokio::time::{Duration, interval};

use phenome_domain::{ScheduleId, ScheduleStatus, ScheduledAction};
use phenome_ports::SchedulerPort;

use crate::storage::StoragePort;

const SCHEDULER_TICK_INTERVAL: Duration = Duration::from_secs(60);
const MAX_ACTIONS_PER_TICK: usize = 64;

#[derive(Clone)]
pub struct SchedulerService {
    storage: Arc<dyn StoragePort>,
}

impl SchedulerService {
    pub fn new(storage: Arc<dyn StoragePort>) -> Self {
        Self { storage }
    }

    pub async fn run_minute(storage: Arc<dyn StoragePort>, kube_client: kube::Client) {
        let (_tx, rx) = watch::channel(false);
        Self::run_minute_with_shutdown(storage, kube_client, rx).await;
    }

    pub async fn run_minute_with_shutdown(
        storage: Arc<dyn StoragePort>,
        kube_client: kube::Client,
        shutdown: watch::Receiver<bool>,
    ) {
        let service = Self::new(storage);
        service.run_scheduler_loop(kube_client, shutdown).await;
    }

    pub async fn run_scheduler_loop(
        &self,
        kube_client: kube::Client,
        mut shutdown: watch::Receiver<bool>,
    ) {
        let mut interval_timer = interval(SCHEDULER_TICK_INTERVAL);
        loop {
            tokio::select! {
                result = shutdown.changed() => {
                    if result.is_err() || *shutdown.borrow() {
                        break;
                    }
                }
                _ = interval_timer.tick() => {
                    if let Err(e) = self.check_and_execute(&kube_client).await {
                        tracing::error!("Scheduler loop error: {}", e);
                    }
                }
            }
        }
    }

    async fn check_and_execute(&self, _kube_client: &kube::Client) -> Result<()> {
        let all = self.storage.get_all_schedules().await?;
        let now = Utc::now().timestamp_millis();

        let mut executed = 0usize;
        for mut action in all {
            // Check if due and pending
            if action.execute_at <= now && matches!(action.status, ScheduleStatus::Pending) {
                if executed >= MAX_ACTIONS_PER_TICK {
                    tracing::warn!(
                        "Scheduler tick exceeded max actions; remaining items deferred"
                    );
                    break;
                }
                // Execute
                tracing::info!("Executing scheduled action: {}", action.id);
                // Mark as Executing
                action.status = ScheduleStatus::Executing;
                self.storage.update_schedule(action.clone()).await?;

                let result = crate::scheduler::executor::execute_action(&action).await;
                if let Err(err) = result {
                    tracing::error!("Scheduled action {} failed: {}", action.id, err);
                    action.status = ScheduleStatus::Failed {
                        error: err.to_string(),
                    };
                } else {
                    action.status = ScheduleStatus::Completed;
                }
                self.storage.update_schedule(action).await?;
                executed += 1;
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
