//! Debug Command
//!
//! ## Responsibility
//! Provides introspection into the Bootstrappo runtime state.
//!
//! ## Subcommands
//! - `registry`: List all registered modules from the component registry.
//! - `assembly-order`: Print the execution order of a given assembly.

/// List all registered modules.
pub async fn registry() -> anyhow::Result<()> {
    println!("=== Registered Modules ===");
    println!();
    for reg in inventory::iter::<bootstrappo::application::runtime::registry::ComponentRegistration>
    {
        let spec = &reg.spec;
        println!("• {} (v{})", spec.name, spec.version);
        println!("  domain:      {}", spec.domain);
        if let Some(ns) = spec.namespace {
            println!("  namespace:   {}", ns);
        }
        if !spec.required.is_empty() {
            println!("  requires:    [{}]", spec.required.join(", "));
        }
        if !spec.description.is_empty() {
            println!("  desc:        {}", spec.description);
        }
        if let Some(url) = spec.url {
            println!("  url:         {}", url);
        }
        println!();
    }
    Ok(())
}

/// Print the execution order of an assembly.
pub async fn assembly_order(assembly: String) -> anyhow::Result<()> {
    let config = bootstrappo::application::config::load_from_file(&assembly)?;
    let modules = bootstrappo::application::runtime::registry::get_all_modules(&config);
    let assembly = bootstrappo::application::composition::assembly::builder::AssemblyBuilder::new(config)
        .with_modules(modules)
        .build()?;
    let steps = assembly.execution_order()?;

    println!("=== Assembly Execution Order ===");
    for (i, step) in steps.iter().enumerate() {
        println!("  {}. {} (kind: {:?})", i + 1, step.id, step.kind);
    }
    Ok(())
}
