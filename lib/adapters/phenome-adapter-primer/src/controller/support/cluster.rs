//! Cluster lifecycle command handlers for bootstrappo CLI commands.

use std::path::Path;
use tracing::info;

use bootstrappo::adapters::infrastructure::kube::cluster::K3sBootstrapConfig;
use bootstrappo::application::events::{EventBus, EventPayload};
use bootstrappo::application::flows::cluster::{detect_existing_cluster, init_cluster_with_events};
use bootstrappo::application::runtime::modules::io::command::CommandAdapter;
use bootstrappo::ports::CommandPort;

const K3S_UNINSTALL_PATH: &str = "/usr/local/bin/k3s-uninstall.sh";

pub async fn init(skip_upgrade: bool, force: bool) -> anyhow::Result<()> {
    let cmd = CommandAdapter::new();
    let config = K3sBootstrapConfig::default();
    let event_bus = EventBus::default();
    let mut progress_rx = event_bus.subscribe();
    let progress_task = tokio::spawn(async move {
        while let Ok(event) = progress_rx.recv().await {
            if let Some(message) = format_k3s_event(&event.payload) {
                println!("{message}");
            }
        }
    });

    if force {
        info!("Force flag set, uninstalling existing cluster.");
        uninstall_k3s(&cmd)?;
        if detect_existing_cluster(&cmd).await?.is_some() {
            anyhow::bail!(
                "Cluster still detected after uninstall. Verify K3s removal before retrying."
            );
        }
    }

    let cluster_info = if skip_upgrade {
        if let Some(existing) = detect_existing_cluster(&cmd).await? {
            existing
        } else {
            init_cluster_with_events(&config, &cmd, Some(&event_bus)).await?
        }
    } else {
        init_cluster_with_events(&config, &cmd, Some(&event_bus)).await?
    };

    drop(event_bus);
    let _ = progress_task.await;

    println!("Cluster initialized successfully");
    println!("Version: {}", cluster_info.version);
    println!("API Server: {}", cluster_info.api_server);
    println!("Kubeconfig: {}", cluster_info.kubeconfig_path.display());
    println!();
    println!("Next steps:");
    println!("  bootstrappo reconcile  # Deploy components");

    Ok(())
}

fn uninstall_k3s(cmd: &dyn CommandPort) -> anyhow::Result<()> {
    if !Path::new(K3S_UNINSTALL_PATH).exists() {
        anyhow::bail!("K3s uninstall script not found at {K3S_UNINSTALL_PATH}");
    }

    cmd.run_capture("sh", &["-c", K3S_UNINSTALL_PATH])?;
    Ok(())
}

fn format_k3s_event(payload: &EventPayload) -> Option<String> {
    match payload {
        EventPayload::K3sDownloadStarted => Some("k3s download started".to_string()),
        EventPayload::K3sDownloadProgress { percent } => {
            Some(format!("k3s download {:.0}%", percent * 100.0))
        }
        EventPayload::K3sDownloadCompleted => Some("k3s download completed".to_string()),
        EventPayload::K3sInstallStarted => Some("k3s install started".to_string()),
        EventPayload::K3sInstallCompleted => Some("k3s install completed".to_string()),
        EventPayload::K3sApiServerReady => Some("k3s API server ready".to_string()),
        EventPayload::K3sCoreDnsReady => Some("k3s CoreDNS ready".to_string()),
        EventPayload::K3sBootstrapCompleted => Some("k3s bootstrap completed".to_string()),
        _ => None,
    }
}
