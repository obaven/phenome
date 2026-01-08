use bootstrappo::application::api::BootstrappoApi;

pub async fn diff() -> anyhow::Result<()> {
    let (config, discovery) = super::status::load_config_and_discovery().await?;
    let api = BootstrappoApi::new();
    let report = api.diff(config.as_ref().clone(), discovery)?;

    println!("=== Diff ===");
    if report.is_converged() {
        println!("Cluster matches expected signals.");
        return Ok(());
    }

    if !report.missing_signals.is_empty() {
        let mut missing = report.missing_signals.clone();
        missing.sort_by(|a, b| a.signal.cmp(&b.signal));
        println!("Missing signals:");
        for item in missing {
            println!(
                "  - {} (provided by {}, reason: {})",
                item.signal, item.provided_by, item.reason
            );
        }
    }

    if !report.unmet_checks.is_empty() {
        let mut unmet = report.unmet_checks.clone();
        unmet.sort_by(|a, b| a.check.cmp(&b.check));
        println!("Unmet checks:");
        for check in unmet {
            let mut blocks = check.blocks.clone();
            blocks.sort();
            println!("  - {} (blocks: {})", check.check, blocks.join(", "));
        }
    }

    if !report.blocked_modules.is_empty() {
        let mut blocked = report.blocked_modules.clone();
        blocked.sort_by(|a, b| a.module.cmp(&b.module));
        println!("Blocked modules:");
        for module in blocked {
            let mut blockers = module.blocked_by.clone();
            blockers.sort();
            println!("  - {} (blocked by: {})", module.module, blockers.join(", "));
        }
    }

    Ok(())
}
