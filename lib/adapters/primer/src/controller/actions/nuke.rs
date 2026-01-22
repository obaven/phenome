use std::io::{self, Write};
use std::process::Command;
use tracing::info;

pub async fn nuke(assembly: String, aggressive: bool, dry_run: bool) -> anyhow::Result<()> {
    info!("☢️  CLUSTER NUKE REQUESTED");

    if dry_run {
        info!("🔍 DRY-RUN MODE: Showing deletion order without executing");
        let config = primer::application::config::load_from_file(&assembly)?;
        let modules = primer::application::runtime::registry::get_all_modules(&config);
        let assembly_data =
            primer::application::composition::assembly::builder::AssemblyBuilder::new(config)
                .with_modules(modules)
                .build()?;
        let steps = assembly_data.execution_order()?;

        println!("=== Nuke Deletion Order (Reverse of Assembly) ===");
        for (i, step) in steps.iter().rev().enumerate() {
            println!("  {}. {} (kind: {:?})", i + 1, step.id, step.kind);
        }
        println!("\n✅ Dry-run complete. No resources were deleted.");
        return Ok(());
    }

    if !aggressive {
        println!("⚠️  WARNING: This will DELETE ALL resources from the cluster!");
        println!("   Assembly: {assembly}");
        print!("   Type 'yes' to confirm: ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        if input.trim().to_lowercase() != "yes" {
            info!("Nuke cancelled by user");
            return Ok(());
        }
    }

    let config = primer::application::config::load_from_file(&assembly)?;
    let modules = primer::application::runtime::registry::get_all_modules(&config);
    let assembly_data =
        primer::application::composition::assembly::builder::AssemblyBuilder::new(config)
            .with_modules(modules)
            .build()?;
    let client = kube::Client::try_default().await?;
    let cmd = std::sync::Arc::new(
        primer::application::runtime::modules::io::command::CommandAdapter::new(),
    );

    // Always use timing for nuke operations (it's useful for diagnostics)
    let options = primer::application::nuke::NukeOptions::default()
        .with_timing()
        .with_timing_output("nuke-timing.json");

    primer::application::nuke::nuke_cluster_with_options(client, &assembly_data, options, cmd)
        .await?;

    // If aggressive, also reset K3s data directory
    if aggressive {
        info!("🔨 AGGRESSIVE MODE: Resetting K3s completely");
        println!("⚠️  This will stop K3s, delete data and manifests, then restart");

        // Stop K3s
        info!("  Stopping K3s...");
        let _ = Command::new("sudo")
            .args(["systemctl", "stop", "k3s"])
            .status();

        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

        // Remove data directory
        info!("  Removing /var/lib/rancher/k3s...");
        let _ = Command::new("sudo")
            .args(["rm", "-rf", "/var/lib/rancher/k3s"])
            .output();

        // Remove manifests directory
        info!("  Removing K3s server manifests...");
        let _ = Command::new("sudo")
            .args(["rm", "-rf", "/var/lib/rancher/k3s/server/manifests"])
            .output();

        // Restart K3s
        info!("  Restarting K3s...");
        let _ = Command::new("sudo")
            .args(["systemctl", "start", "k3s"])
            .status();

        info!("  Waiting for K3s to start...");
        tokio::time::sleep(tokio::time::Duration::from_secs(15)).await;

        // Clean up any auto-deployed resources that K3s created on startup
        info!("  Removing auto-deployed HelmCharts...");
        let _ = Command::new("kubectl")
            .args([
                "delete",
                "helmcharts.helm.cattle.io",
                "--all",
                "-A",
                "--wait=false",
            ])
            .output();

        info!("  Removing auto-deployed manifests...");
        let _ = Command::new("sudo")
            .args(["rm", "-rf", "/var/lib/rancher/k3s/server/manifests/*"])
            .output();

        info!("✅ K3s reset complete.");
    }
    Ok(())
}
