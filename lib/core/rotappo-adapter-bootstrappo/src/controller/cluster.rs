use std::path::Path;
use tracing::info;

use bootstrappo::adapters::infrastructure::kube::cluster::K3sBootstrapConfig;
use bootstrappo::application::cluster::{detect_existing_cluster, init_cluster};
use bootstrappo::application::runtime::modules::io::command::CommandAdapter;
use bootstrappo::ports::CommandPort;

const K3S_UNINSTALL_PATH: &str = "/usr/local/bin/k3s-uninstall.sh";

pub async fn init(skip_upgrade: bool, force: bool) -> anyhow::Result<()> {
    let cmd = CommandAdapter::new();
    let config = K3sBootstrapConfig::default();

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
            init_cluster(&config, &cmd).await?
        }
    } else {
        init_cluster(&config, &cmd).await?
    };

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
        anyhow::bail!("K3s uninstall script not found at {}", K3S_UNINSTALL_PATH);
    }

    cmd.run_capture("sh", &["-c", K3S_UNINSTALL_PATH])?;
    Ok(())
}
