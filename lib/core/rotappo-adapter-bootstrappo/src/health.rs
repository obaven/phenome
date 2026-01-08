use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::Duration;

use bootstrappo::application::runtime::modules::runtime::k8s::cache::ClusterCache;
use bootstrappo::application::runtime::registry;
use bootstrappo::ports::module::{HealthStatus, ModuleContext, ModuleMode};
use bootstrappo_api::contract::config::Config;
use kube::Client;

use rotappo_domain::{ComponentHealthStatus, HealthSnapshot};
use rotappo_ports::HealthPort;

#[derive(Clone)]
pub struct LiveStatus {
    cache: Arc<RwLock<Option<ClusterCache>>>,
    health: Arc<RwLock<HashMap<String, HealthStatus>>>,
    error: Arc<RwLock<Option<String>>>,
}

impl LiveStatus {
    pub fn spawn(config: Arc<Config>) -> Self {
        let live = Self {
            cache: Arc::new(RwLock::new(None)),
            health: Arc::new(RwLock::new(HashMap::new())),
            error: Arc::new(RwLock::new(None)),
        };

        if std::env::var("ROTAPPO_DISABLE_LIVE_STATUS")
            .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
            .unwrap_or(false)
        {
            return live;
        }

        let cache = Arc::clone(&live.cache);
        let health = Arc::clone(&live.health);
        let error = Arc::clone(&live.error);

        thread::spawn(move || {
            let runtime = tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build();

            let runtime = match runtime {
                Ok(rt) => rt,
                Err(err) => {
                    if let Ok(mut guard) = error.write() {
                        *guard = Some(format!("Failed to build tokio runtime: {}", err));
                    }
                    return;
                }
            };

            runtime.block_on(async move {
                if let Err(err) = init_cache(&cache).await {
                    if let Ok(mut guard) = error.write() {
                        *guard = Some(err);
                    }
                }

                let mut interval = tokio::time::interval(Duration::from_secs(15));
                loop {
                    interval.tick().await;
                    let ctx = ModuleContext::new(Arc::clone(&config), ModuleMode::Render);
                    let modules = registry::get_all_modules(config.as_ref());
                    let mut results = HashMap::new();

                    for module in modules {
                        if !module.enabled(&ctx) {
                            continue;
                        }
                        let name = module.spec().name.to_string();
                        let status = match module.check().await {
                            Ok(status) => status,
                            Err(err) => HealthStatus::Unhealthy(err.to_string()),
                        };
                        results.insert(name, status);
                    }

                    if let Ok(mut guard) = health.write() {
                        *guard = results;
                    }
                }
            });
        });

        live
    }

    pub fn cache(&self) -> Option<ClusterCache> {
        self.cache.read().ok().and_then(|guard| guard.clone())
    }

    pub fn health(&self) -> HashMap<String, HealthStatus> {
        self.health
            .read()
            .map(|guard| guard.clone())
            .unwrap_or_default()
    }

    pub fn last_error(&self) -> Option<String> {
        self.error.read().ok().and_then(|guard| guard.clone())
    }
}

#[derive(Clone)]
pub struct BootstrappoHealthPort {
    live_status: Option<LiveStatus>,
}

impl BootstrappoHealthPort {
    pub fn new(live_status: Option<LiveStatus>) -> Self {
        Self { live_status }
    }
}

impl HealthPort for BootstrappoHealthPort {
    fn snapshot(&self) -> HealthSnapshot {
        let Some(live) = &self.live_status else {
            return HealthSnapshot::default();
        };
        let raw = live.health();
        let mut health = HashMap::new();
        for (name, status) in raw {
            health.insert(name, map_health_status(status));
        }
        HealthSnapshot {
            health,
            last_error: live.last_error(),
            cache_ready: live.cache().is_some(),
        }
    }
}

fn map_health_status(status: HealthStatus) -> ComponentHealthStatus {
    match status {
        HealthStatus::Healthy => ComponentHealthStatus::Healthy,
        HealthStatus::Degraded(msg) => ComponentHealthStatus::Degraded(msg),
        HealthStatus::Unhealthy(msg) => ComponentHealthStatus::Unhealthy(msg),
    }
}

async fn init_cache(cache: &Arc<RwLock<Option<ClusterCache>>>) -> Result<(), String> {
    let client = Client::try_default()
        .await
        .map_err(|err| format!("Failed to init kube client: {}", err))?;
    let cluster_cache = ClusterCache::new(client);
    cluster_cache.start_watchers().await;
    if let Ok(mut guard) = cache.write() {
        *guard = Some(cluster_cache.clone());
    }
    Ok(())
}
