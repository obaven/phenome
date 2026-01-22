use anyhow::Result;

use phenome_domain::Notification;

pub async fn send(_notification: &Notification, _endpoint: &str) -> Result<()> {
    Ok(())
}
