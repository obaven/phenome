use primer::application::api::BootstrappoApi;
use primer::application::flows::reconcile::ops::hooks::registry::all_hook_specs;
use primer::domain::models::module::EngineMeta;
use primer::domain::models::signal::SignalRef;

pub async fn explain(module: String) -> anyhow::Result<()> {
    primer::application::config::load()?;
    let config = primer::application::config::instance();
    let api = BootstrappoApi::new();

    let spec = api
        .explain(config.as_ref().clone(), &module)
        .ok_or_else(|| anyhow::anyhow!("Module '{module}' is not registered"))?;

    println!("=== Module: {} ===", spec.name);
    println!("Domain: {}", spec.domain);
    println!("Kind: {}", spec.kind.to_string().to_lowercase());
    println!("Version: {}", spec.version);
    if let Some(ns) = spec.namespace {
        println!("Namespace: {ns}");
    }
    if !spec.maintainer.is_empty() {
        println!("Maintainer: {}", spec.maintainer);
    }
    if !spec.description.is_empty() {
        println!("Description: {}", spec.description);
    }
    if let Some(url) = spec.url {
        println!("URL: {url}");
    }

    if spec.required.is_empty() {
        println!("Requires: none");
    } else {
        println!("Requires: {}", spec.required.join(", "));
    }

    if spec.provides.is_empty() {
        println!("Provides: none");
    } else {
        let provides_str: Vec<_> = spec.provides.iter().map(|s| s.as_str()).collect();
        println!("Provides: {}", provides_str.join(", "));
    }

    if spec.checks.is_empty() {
        println!("Checks: none");
    } else {
        println!("Checks:");
        for check in spec.checks {
            println!("  - {check}");
        }
    }

    print_engine(&spec.engine);
    show_hooks(spec.provides);

    Ok(())
}

fn print_engine(engine: &Option<EngineMeta>) {
    match engine {
        Some(EngineMeta::Helm(meta)) => {
            println!("Engine: helm");
            println!("  repo: {}", meta.repo);
            println!("  chart: {}", meta.chart);
            println!("  version: {}", meta.version);
            println!("  namespace: {}", meta.namespace);
            println!("  release: {}", meta.release);
            if let Some(values) = meta.values_template {
                println!("  values_template: {values}");
            }
        }
        Some(EngineMeta::Kro(meta)) => {
            println!("Engine: kro");
            println!("  rgd_template: {}", meta.rgd_template);
            println!("  instance_template: {}", meta.instance_template);
        }
        Some(EngineMeta::Terraform(meta)) => {
            println!("Engine: terraform");
            println!("  template_path: {}", meta.template_path);
            if !meta.secrets.is_empty() {
                println!("  secrets: {}", meta.secrets.join(", "));
            }
        }
        Some(EngineMeta::Native) | None => {
            println!("Engine: native");
        }
    }
}

fn show_hooks(provides: &[SignalRef]) {
    let mut hooks = all_hook_specs();
    hooks.sort_by(|a, b| a.id.cmp(&b.id));
    let mut matches = Vec::new();

    for hook in hooks {
        if provides
            .iter()
            .any(|capability| hook.capability_required == capability.as_str())
        {
            matches.push(hook);
        }
    }

    if matches.is_empty() {
        println!("Hooks: none");
        return;
    }

    println!("Hooks:");
    for hook in matches {
        println!(
            "  - {} (action: {}, requires: {}, timing: {:?}, priority: {}, idempotent: {})",
            hook.id,
            hook.action.display_name(),
            hook.capability_required,
            hook.timing,
            hook.priority,
            hook.idempotent
        );
    }
}
