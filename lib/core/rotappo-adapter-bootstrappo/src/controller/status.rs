use std::collections::HashSet;
use std::sync::Arc;
use std::time::Duration;

use bootstrappo::application::api::BootstrappoApi;
use bootstrappo::application::flows::reconcile::core::assembly::validation::{
    Severity, ValidationReport,
};
use bootstrappo::application::runtime::modules::runtime::k8s::cache::ClusterCache;
use bootstrappo::ports::discovery::ClusterDiscoveryPort;

pub async fn status() -> anyhow::Result<()> {
    let (config, discovery) = load_config_and_discovery().await?;
    let api = BootstrappoApi::new();
    let report = api.status(config.as_ref().clone(), discovery)?;

    println!("=== Status ===");
    println!("Modules: {}", report.module_count);
    println!(
        "Checks: {}/{} satisfied",
        report.satisfied_check_count(),
        report.check_results.len()
    );
    println!("Observed signals: {}", report.observed_signals.len());

    print_validation(&report.validation);
    print_signals(&report.observed_signals);
    print_checks(&report.check_results);

    Ok(())
}

pub(crate) async fn load_config_and_discovery(
) -> anyhow::Result<(
    Arc<bootstrappo::application::config::Config>,
    Arc<dyn ClusterDiscoveryPort>,
)> {
    bootstrappo::application::config::load()?;
    let config = bootstrappo::application::config::instance();

    let client = kube::Client::try_default().await?;
    let discovery = Arc::new(ClusterCache::new(client));
    discovery.start_watchers().await;
    tokio::time::sleep(Duration::from_secs(2)).await;

    Ok((config, discovery))
}

fn print_validation(report: &ValidationReport) {
    if report.errors.is_empty() {
        println!("Validation: ok");
        return;
    }

    let error_count = report
        .errors
        .iter()
        .filter(|err| err.severity() == Severity::Error)
        .count();
    let warning_count = report
        .errors
        .iter()
        .filter(|err| err.severity() == Severity::Warning)
        .count();

    println!(
        "Validation: {} error(s), {} warning(s)",
        error_count, warning_count
    );
    for err in &report.errors {
        let severity = match err.severity() {
            Severity::Error => "error",
            Severity::Warning => "warn",
        };
        println!("  [{}] {} {}", severity, err.code(), err);
    }
}

fn print_signals(observed: &HashSet<String>) {
    if observed.is_empty() {
        println!("Observed signals: none");
        return;
    }

    let mut signals: Vec<_> = observed.iter().cloned().collect();
    signals.sort();

    println!("Observed signals:");
    for signal in signals {
        println!("  - {}", signal);
    }
}

fn print_checks(checks: &[bootstrappo::application::api::status::CheckResult]) {
    if checks.is_empty() {
        println!("Checks: none");
        return;
    }

    let mut results = checks.to_vec();
    results.sort_by(|a, b| a.check.cmp(&b.check));

    println!("Checks:");
    for result in results {
        let status = if result.satisfied { "ok" } else { "fail" };
        let required_by = result
            .required_by
            .as_deref()
            .unwrap_or("unknown");
        println!("  [{}] {} (required by {})", status, result.check, required_by);
    }
}
