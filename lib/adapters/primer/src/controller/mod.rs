pub mod actions;
pub mod ops;
pub mod support;

pub use actions::{generate, nuke, rotate, visualize};
pub use ops::{assembly, diff, explain, reconcile, status};
pub use support::{cache, catalog, cluster, debug};

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
    if let Err(err) = primer::application::config::load() {
        return BootstrappoConfigSummary {
            host_domain: "unknown".to_string(),
            metallb_pools: Vec::new(),
            load_error: Some(err.to_string()),
        };
    }
    let config = primer::application::config::instance();
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
