use crate::ports::CachePort;

use super::health::LiveStatus;

#[derive(Clone)]
pub struct BootstrappoCachePort {
    live_status: Option<LiveStatus>,
}

impl BootstrappoCachePort {
    pub fn new(live_status: Option<LiveStatus>) -> Self {
        Self { live_status }
    }
}

impl CachePort for BootstrappoCachePort {
    fn cache(&self) -> Option<bootstrappo::ops::k8s::cache::ClusterCache> {
        self.live_status
            .as_ref()
            .and_then(|live| live.cache())
    }
}
