use anyhow::Result;
use bootstrappo_api::contract::config::core::system::{StorageSystemConfig, SystemConfig};
use tracing::info;

#[derive(Debug, Clone)]
pub struct StorageArgs {
    pub min_size: Option<u64>,
}

/// Generate storage configuration by scanning system devices.
pub async fn storage(args: StorageArgs) -> Result<()> {
    info!("Scanning for storage devices...");
    let devices =
        bootstrappo::adapters::infrastructure::kube::discovery::storage::scan_block_devices(
            args.min_size,
        )?;

    if devices.is_empty() {
        info!("No suitable storage devices found.");
        return Ok(());
    }

    // Create a partial SystemConfig to serialize
    let config = SystemConfig {
        storage: StorageSystemConfig { devices },
    };

    println!("{}", serde_yaml::to_string(&config)?);

    // Hint for visualization
    info!("To visualize, you can pipe this output or manually update your config.");

    Ok(())
}
