use anyhow::Result;

use rotappo_domain::Notification;

pub async fn send(_notification: &Notification, _url: &str) -> Result<()> {
    Ok(())
}
