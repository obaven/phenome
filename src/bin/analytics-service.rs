use std::env;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::watch;

use rotappo_adapter_analytics::AnalyticsService;
use rotappo_adapter_analytics::cluster_manager::ClusterManager;
use rotappo_adapter_analytics::grpc::GrpcServer;
use rotappo_adapter_analytics::storage::sqlite::{RetentionConfig, SqliteStorage};
use phenome_domain::RotappoConfig;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config_path = config_path();
    let config = RotappoConfig::load_from_path(&config_path)?;

    let (shutdown_tx, shutdown_rx) = watch::channel(false);
    let shutdown_signal = shutdown_tx.clone();
    tokio::spawn(async move {
        if tokio::signal::ctrl_c().await.is_ok() {
            let _ = shutdown_signal.send(true);
        }
    });

    let retention = RetentionConfig {
        raw_days: config.analytics.retention.full_resolution_days,
        aggregated_days: config.analytics.retention.aggregated_days,
    };
    let storage = Arc::new(SqliteStorage::with_retention(
        &config.analytics.sqlite_path,
        retention,
    )?);

    let ml_url = config.services.ml_url.clone();
    let ml_client = rotappo_adapter_analytics::grpc::MlClient::connect(&ml_url).await?;

    let service = AnalyticsService::new(storage.clone(), ml_client);
    let service = Arc::new(service);

    let cm = ClusterManager::new();
    for cluster_config in config.clusters {
        cm.add_cluster(cluster_config.context).await?;
    }
    let mc = rotappo_adapter_analytics::metrics_collector::MetricsCollector::new(
        cm,
        Duration::from_secs(config.collection.interval),
    );
    let _hc = tokio::spawn(mc.run_polling_loop_with_shutdown(shutdown_rx.clone()));

    tokio::spawn(
        rotappo_adapter_analytics::aggregator::Aggregator::run_hourly_with_shutdown(
            storage.clone(),
            shutdown_rx.clone(),
        ),
    );

    let kube_client = match kube::Client::try_default().await {
        Ok(client) => Some(client),
        Err(err) => {
            tracing::warn!(
                "Failed to create kube client; scheduler disabled: {}",
                err
            );
            None
        }
    };

    let mut channels = Vec::new();
    channels.push(phenome_domain::NotificationChannel::InTui);
    channels.push(phenome_domain::NotificationChannel::System);

    for channel_config in config.notifications.channels {
        match channel_config {
            phenome_domain::NotificationChannelConfig::Ntfy { url, topic } => {
                channels.push(phenome_domain::NotificationChannel::Ntfy { url, topic });
            }
        }
    }

    let notifier =
        Arc::new(rotappo_adapter_analytics::notification::NotificationService::new(channels));
    {
        let notifier = notifier.clone();
        let service = service.clone();
        let shutdown_rx = shutdown_rx.clone();
        tokio::spawn(async move {
            notifier
                .watch_anomalies_with_shutdown(service, shutdown_rx)
                .await;
        });
    }

    if let Some(kube_client) = kube_client {
        tokio::spawn(
            rotappo_adapter_analytics::scheduler::SchedulerService::run_minute_with_shutdown(
                storage.clone(),
                kube_client,
                shutdown_rx.clone(),
            ),
        );
    }

    let addr = parse_addr(&config.services.analytics_url)
        .unwrap_or_else(|| "127.0.0.1:50051".parse().expect("invalid fallback addr"));
    GrpcServer::serve(addr, service).await?;
    Ok(())
}

fn config_path() -> PathBuf {
    if let Ok(path) = env::var("ROTAPPO_CONFIG_PATH") {
        return PathBuf::from(path);
    }

    if let Ok(home) = env::var("HOME") {
        return Path::new(&home).join(".rotappo").join("config.yaml");
    }

    PathBuf::from("rotappo-config.yaml")
}

fn parse_addr(raw: &str) -> Option<SocketAddr> {
    let trimmed = raw.trim();
    let value = trimmed
        .strip_prefix("http://")
        .or_else(|| trimmed.strip_prefix("https://"))
        .unwrap_or(trimmed);
    value.parse().ok()
}
