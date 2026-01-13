use anyhow::Result;
use std::net::SocketAddr;
use std::sync::Arc;
use tonic::{Request, Response, Status};

use crate::NotificationService;
use rotappo_domain as domain;
use rotappo_ports::NotificationPort;

pub mod analytics {
    tonic::include_proto!("analytics");
}

pub mod notification {
    tonic::include_proto!("notification");
}

use notification::notification_service_server::{
    NotificationService as NotificationServiceTrait, NotificationServiceServer,
};
use notification::*;

#[derive(Debug)]
pub struct GrpcNotificationService {
    inner: Arc<NotificationService>,
}

impl GrpcNotificationService {
    pub fn new(inner: Arc<NotificationService>) -> Self {
        Self { inner }
    }
}

#[tonic::async_trait]
impl NotificationServiceTrait for GrpcNotificationService {
    async fn send_notification(
        &self,
        request: Request<SendNotificationRequest>,
    ) -> Result<Response<SendNotificationResponse>, Status> {
        let req = request.into_inner();
        let notification: domain::Notification = req
            .notification
            .ok_or_else(|| Status::invalid_argument("missing notification"))?
            .try_into()
            .map_err(|e: anyhow::Error| Status::invalid_argument(e.to_string()))?;

        self.inner
            .send_notification(notification)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(SendNotificationResponse { success: true }))
    }

    async fn configure_channel(
        &self,
        request: Request<ConfigureChannelRequest>,
    ) -> Result<Response<ConfigureChannelResponse>, Status> {
        let req = request.into_inner();
        let channel: domain::NotificationChannel = req
            .channel
            .ok_or_else(|| Status::invalid_argument("missing channel"))?
            .try_into()
            .map_err(|e: anyhow::Error| Status::invalid_argument(e.to_string()))?;

        self.inner
            .configure_channel(channel)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(ConfigureChannelResponse { success: true }))
    }
}

pub struct GrpcServer;

impl GrpcServer {
    pub async fn serve(addr: SocketAddr, service: Arc<NotificationService>) -> Result<()> {
        let grpc_service = GrpcNotificationService::new(service);
        tonic::transport::Server::builder()
            .add_service(NotificationServiceServer::new(grpc_service))
            .serve(addr)
            .await?;
        Ok(())
    }
}

// Conversions

impl TryFrom<notification::Notification> for domain::Notification {
    type Error = anyhow::Error;

    fn try_from(val: notification::Notification) -> Result<Self, Self::Error> {
        Ok(domain::Notification {
            id: val.id,
            title: val.title,
            message: val.message,
            severity: analytics::Severity::try_from(val.severity)
                .map_err(|_| anyhow::anyhow!("invalid severity"))?
                .try_into()?,
            timestamp: val.timestamp,
            read: val.read,
            link: val.link,
            cluster_id: val.cluster_id,
            resource_id: val.resource_id,
        })
    }
}

impl TryFrom<notification::NotificationChannel> for domain::NotificationChannel {
    type Error = anyhow::Error;

    fn try_from(val: notification::NotificationChannel) -> Result<Self, Self::Error> {
        // Serialize config from oneof into json
        let config_json = match &val.config {
            Some(notification::notification_channel::Config::InTui(v)) => {
                serde_json::json!({ "type": "tui", "enabled": v }).to_string()
            }
            Some(notification::notification_channel::Config::System(v)) => {
                serde_json::json!({ "type": "system", "enabled": v }).to_string()
            }
            Some(notification::notification_channel::Config::Ntfy(n)) => {
                serde_json::json!({ "type": "ntfy", "url": n.url, "topic": n.topic }).to_string()
            }
            Some(notification::notification_channel::Config::Webhook(w)) => {
                serde_json::json!({ "type": "webhook", "url": w.url, "headers": w.headers })
                    .to_string()
            }
            None => "{}".to_string(),
        };

        Ok(domain::NotificationChannel {
            id: val.id,
            name: val.name,
            enabled: val.enabled,
            config_json,
            ..Default::default()
        })
    }
}

impl TryFrom<analytics::Severity> for domain::Severity {
    type Error = anyhow::Error;

    fn try_from(val: analytics::Severity) -> Result<Self, Self::Error> {
        match val {
            analytics::Severity::Critical => Ok(domain::Severity::Critical),
            analytics::Severity::Warning => Ok(domain::Severity::Warning),
            analytics::Severity::Info => Ok(domain::Severity::Info),
            analytics::Severity::Unspecified => Ok(domain::Severity::Info), // Fallback
        }
    }
}
