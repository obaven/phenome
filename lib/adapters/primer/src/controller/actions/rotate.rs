use std::path::Path;
use tracing::info;

pub async fn rotate(
    rotation: String,
    assembly_path: String,
    gitops_dir: String,
    dry_run: bool,
) -> anyhow::Result<()> {
    info!("Triggering rotation: {}", rotation);

    if dry_run {
        info!("🔍 DRY-RUN MODE: Preview of rotation targets");
        println!("=== Rotation Preview ===");
        println!("  Type: {rotation}");
        println!("  Assembly: {assembly_path}");
        println!("  GitOps Dir: {gitops_dir}");
        println!("\n✅ Dry-run complete. No rotations were executed.");
        return Ok(());
    }

    let config = primer::application::config::load_from_file(&assembly_path)?;
    let modules = primer::application::runtime::registry::get_all_modules(&config);
    let action_data =
        primer::application::composition::assembly::builder::AssemblyBuilder::new(config)
            .with_modules(modules)
            .build()?;
    let fs = std::sync::Arc::new(
        primer::adapters::infrastructure::core::filesystem::RealFilesystemAdapter::new(),
    );
    let gitops_dir = Path::new(&gitops_dir);

    use primer::application::reconciler::ops::hooks::{
        HookOperation, dns::DnsHook, ingress::IngressHook, tls::TlsHook,
    };

    match rotation.as_str() {
        "ingress" => {
            info!("Running ingress hook...");
            let hook = IngressHook;
            hook.execute(&action_data, gitops_dir, fs.clone()).await?;
            info!("✅ Ingress hook complete!");
        }
        "tls" => {
            info!("Running TLS hook...");
            let hook = TlsHook;
            hook.execute(&action_data, gitops_dir, fs.clone()).await?;
            info!("✅ TLS hook complete!");
        }
        "dns" => {
            info!("Running DNS hook...");
            let hook = DnsHook;
            hook.execute(&action_data, gitops_dir, fs.clone()).await?;
            info!("✅ DNS hook complete!");
        }
        "all" => {
            info!("Running all hooks...");
            let hooks: Vec<Box<dyn HookOperation>> =
                vec![Box::new(IngressHook), Box::new(TlsHook), Box::new(DnsHook)];
            for hook in hooks {
                hook.execute(&action_data, gitops_dir, fs.clone()).await?;
            }
            info!("✅ All hooks complete!");
        }
        other => {
            info!("⚠️ Rotation '{other}' not yet implemented");
            anyhow::bail!(
                "Rotation '{other}' not yet implemented. Available: ingress, tls, dns, all"
            );
        }
    }
    Ok(())
}
