use bootstrappo::application::api::BootstrappoApi;
use bootstrappo::application::flows::reconcile::ops::hooks::registry::all_hook_specs;
use bootstrappo::domain::models::module::EngineMeta;

pub async fn explain(module: String) -> anyhow::Result<()> {
    bootstrappo::application::config::load()?;
    let config = bootstrappo::application::config::instance();
    let api = BootstrappoApi::new();

    let spec = api
        .explain(config.as_ref().clone(), &module)
        .ok_or_else(|| anyhow::anyhow!("Module '{}' is not registered", module))?;

    println!("=== Module: {} ===", spec.name);
    println!("Domain: {}", spec.domain);
    println!("Kind: {}", spec.kind.to_string().to_lowercase());
    println!("Version: {}", spec.version);
    if let Some(ns) = spec.namespace {
        println!("Namespace: {}", ns);
    }
    if !spec.maintainer.is_empty() {
        println!("Maintainer: {}", spec.maintainer);
    }
    if !spec.description.is_empty() {
        println!("Description: {}", spec.description);
    }
    if let Some(url) = spec.url {
        println!("URL: {}", url);
    }

    if spec.required.is_empty() {
        println!("Requires: none");
    } else {
        println!("Requires: {}", spec.required.join(", "));
    }

    if spec.provides.is_empty() {
        println!("Provides: none");
    } else {
        println!("Provides: {}", spec.provides.join(", "));
    }

    if spec.checks.is_empty() {
        println!("Checks: none");
    } else {
        println!("Checks:");
        for check in spec.checks {
            println!("  - {}", check);
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
                println!("  values_template: {}", values);
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

fn show_hooks(provides: &[&'static str]) {
    let mut hooks = all_hook_specs();
    hooks.sort_by(|a, b| a.id.cmp(&b.id));
    let mut matches = Vec::new();

    for hook in hooks {
        if provides
            .iter()
            .any(|capability| hook.capability_required == *capability)
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
