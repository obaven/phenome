use anyhow::Result;

use rotappo_domain::Notification;

pub async fn send(_notification: &Notification) -> Result<()> {
    Ok(())
}
