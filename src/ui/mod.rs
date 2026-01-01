use anyhow::Result;

use crate::adapters::bootstrappo::BootstrappoBackend;

pub fn start() -> Result<()> {
    let backend = BootstrappoBackend::from_env()?;
    println!(
        "Rotappo UI connected to Bootstrappo (config={})",
        backend.config_path.display()
    );
    println!(
        "Bootstrappo host domain: {}",
        backend.config.network.host_domain
    );
    Ok(())
}
