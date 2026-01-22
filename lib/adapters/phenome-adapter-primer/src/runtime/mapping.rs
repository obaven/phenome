use std::collections::HashMap;

use bootstrappo::application::runtime::modules::runtime::k8s::cache::ClusterCache;
use bootstrappo_api::contract::assembly::{Check, Step};
use bootstrappo_api::contract::config::Config;
use validator::Validate;

pub fn module_specs() -> HashMap<String, (String, Option<String>)> {
    bootstrappo::application::runtime::registry::get_all_specs()
        .into_iter()
        .map(|spec| {
            (
                spec.name.to_string(),
                (
                    spec.domain.to_string(),
                    spec.namespace.map(|namespace| namespace.to_string()),
                ),
            )
        })
        .collect()
}

pub fn derive_pod_value(step: &Step, namespace: Option<&str>) -> Option<String> {
    let check_label = step.checks.iter().find_map(|check| match check {
        Check::DaemonsetReady { namespace, name } => Some(format!("{namespace}/{name}")),
        Check::DeploymentReady { namespace, name } => Some(format!("{namespace}/{name}")),
        Check::StatefulsetReady { namespace, name } => Some(format!("{namespace}/{name}")),
        Check::SecretExists { namespace, name } => Some(format!("{namespace}/{name}")),
        Check::JobReady { namespace, name } => Some(format!("{namespace}/{name}")),
        Check::ServiceReady { namespace, name } => Some(format!("{namespace}/{name}")),
        _ => None,
    });

    check_label.or_else(|| namespace.map(|ns| format!("{ns}/{id}", id = step.id)))
}

pub fn checks_ready(cache: &ClusterCache, step: &Step, config: Option<&Config>) -> bool {
    if step.checks.is_empty() {
        return true;
    }
    for check in &step.checks {
        match check {
            Check::DaemonsetReady { namespace, name } => {
                if !cache.is_daemonset_ready(namespace, name) {
                    return false;
                }
            }
            Check::DeploymentReady { namespace, name } => {
                if !cache.is_deployment_ready(namespace, name) {
                    return false;
                }
            }
            Check::StatefulsetReady { namespace, name } => {
                if !cache.is_statefulset_ready(namespace, name) {
                    return false;
                }
            }
            Check::CrdEstablished { name } => {
                if !cache.is_crd_established(name) {
                    return false;
                }
            }
            Check::SecretExists { namespace, name } => {
                if !cache.is_secret_ready(namespace, name) {
                    return false;
                }
            }
            Check::JobReady { namespace, name } => {
                if !cache.is_job_ready(namespace, name) {
                    return false;
                }
            }
            Check::ServiceReady { namespace, name } => {
                if !cache.is_service_ready(namespace, name) {
                    return false;
                }
            }
            Check::OidcValid => {
                let Some(config) = config else {
                    return false;
                };
                if !config.security.authelia.oidc.enabled {
                    return false;
                }
                if config.security.authelia.validate().is_err() {
                    return false;
                }
            }
        }
    }
    true
}
