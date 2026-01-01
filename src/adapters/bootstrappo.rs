use anyhow::{Context, Result};
use std::path::PathBuf;
use std::sync::Arc;

pub struct BootstrappoBackend {
    pub config: Arc<bootstrappo::config::Config>,
    pub config_path: PathBuf,
}

impl BootstrappoBackend {
    pub fn from_env() -> Result<Self> {
        let config_path = std::env::var("BOOTSTRAPPO_CONFIG_PATH")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                PathBuf::from("../bootstrappo/data/configs/bootstrap-config.yaml")
            });

        let config = bootstrappo::config::load_from_file(&config_path).with_context(|| {
            format!(
                "Failed to load Bootstrappo config at {}",
                config_path.display()
            )
        })?;

        Ok(Self {
            config: Arc::new(config),
            config_path,
        })
    }
}
