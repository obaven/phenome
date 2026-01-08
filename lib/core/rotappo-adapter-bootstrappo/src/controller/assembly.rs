use tracing::info;

use bootstrappo::application::flows::reconcile::core::assembly::validation;
use bootstrappo::application::composition::assembly::builder::AssemblyBuilder;

#[derive(Debug, Clone)]
pub struct AssemblyVisualizeArgs {
    pub path: String,
    pub view: String,
    pub format: String,
    pub layout: String,
    pub output: Option<String>,
}

/// Validate an assembly config file.
pub async fn validate(path: String) -> anyhow::Result<()> {
    info!("Validating config: {}", path);
    let config = bootstrappo::application::config::load_from_file(&path)?;
    let modules = bootstrappo::application::runtime::registry::get_all_modules(&config);
    let assembly = AssemblyBuilder::new(config)
        .with_modules(modules)
        .build()?;

    let report = validation::rules::validate_with_diagnostics(&assembly);
    if report.has_diagnostics() {
        for error in &report.errors {
            let severity = error.severity();
            let code = error.code();
            info!("[{:?}] {} {}", severity, code, error);
        }
    }

    if report.has_errors() {
        anyhow::bail!("Assembly validation failed");
    }
    info!("✅ Assembly is valid!");
    info!("  Version: {}", assembly.version);
    info!("  Steps: {}", assembly.steps.len());
    Ok(())
}

/// Generate a visualization of the assembly.
pub async fn visualize(args: AssemblyVisualizeArgs) -> anyhow::Result<()> {
    let config = bootstrappo::application::config::load_from_file(&args.path)?;
    let modules = bootstrappo::application::runtime::registry::get_all_modules(&config);
    let assembly = AssemblyBuilder::new(config)
        .with_modules(modules)
        .build()?;

    use bootstrappo::application::reconciler::visualize::{LayerType, OutputFormat, ViewType};

    let view_type = match args.view.to_lowercase().as_str() {
        "full" => ViewType::Full,
        "storage" => ViewType::Storage,
        "network" => ViewType::Network,
        "dependencies" | "deps" => ViewType::Dependencies,
        "registry" => ViewType::Registry,
        "dual" => ViewType::Dual,
        "gitops" => ViewType::Layer(LayerType::GitOps),
        "security" => ViewType::Layer(LayerType::Security),
        "system" => ViewType::Layer(LayerType::System),
        other => {
            anyhow::bail!("Unknown view type: {}", other);
        }
    };

    let output_format = match args.format.to_lowercase().as_str() {
        "svg" => OutputFormat::Svg,
        "dot" => OutputFormat::Dot,
        "png" => OutputFormat::Png,
        other => anyhow::bail!("Unknown format: {}", other),
    };

    // Auto-generate output path if not specified
    let output_path = args.output.unwrap_or_else(|| {
        let ext = match output_format {
            OutputFormat::Svg => "svg",
            OutputFormat::Dot => "dot",
            OutputFormat::Png => "png",
        };
        format!("data/output/diagrams/assembly-{}.{}", args.view.to_lowercase(), ext)
    });

    // Ensure output directory exists
    if let Some(parent) = std::path::Path::new(&output_path).parent() {
        std::fs::create_dir_all(parent)?;
    }

    let visualizer = std::sync::Arc::new(
        bootstrappo::application::runtime::modules::support::visualizer::VisualizerAdapter::new(),
    );

    info!("Generating {} view → {}", args.view, output_path);
    bootstrappo::application::reconciler::visualize::generate(
        &assembly,
        view_type,
        output_format,
        &args.layout,
        Some(output_path.as_str()),
        visualizer,
    )
    .await?;
    Ok(())
}
