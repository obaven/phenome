pub mod cache;
pub mod catalog;
pub mod debug;
pub mod diff;
pub mod explain;
pub mod generate;
pub mod nuke;
pub mod assembly;
pub mod reconcile;
pub mod rotate;
pub mod status;
pub mod visualize;

#[derive(Debug, Clone)]
pub struct MetallbPoolSummary {
    pub name: String,
    pub ip_range: String,
}

#[derive(Debug, Clone)]
pub struct BootstrappoConfigSummary {
    pub host_domain: String,
    pub metallb_pools: Vec<MetallbPoolSummary>,
    pub load_error: Option<String>,
}

pub fn load_config_summary() -> BootstrappoConfigSummary {
    if let Err(err) = bootstrappo::application::config::load() {
        return BootstrappoConfigSummary {
            host_domain: "unknown".to_string(),
            metallb_pools: Vec::new(),
            load_error: Some(err.to_string()),
        };
    }
    let config = bootstrappo::application::config::instance();
    let metallb_pools = config
        .network
        .metallb
        .pools
        .iter()
        .map(|pool| MetallbPoolSummary {
            name: pool.name.clone(),
            ip_range: pool.ip_range.clone(),
        })
        .collect::<Vec<_>>();

    BootstrappoConfigSummary {
        host_domain: config.network.host_domain.clone(),
        metallb_pools,
        load_error: None,
    }
}
