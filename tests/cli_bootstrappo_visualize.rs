#![cfg(all(feature = "cli", feature = "module-bootstrappo"))]

use std::path::{Path, PathBuf};

use phenome_adapter_primer::controller as adapter_controller;
use rotappo_ui_terminal::cli::bootstrappo::{AssemblyAction, Commands};

fn fixture_assembly_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("bootstrappo")
        .join("assembly-minimal.yaml")
}

#[tokio::test]
async fn bootstrappo_cli_assembly_visualize_smoke() -> anyhow::Result<()> {
    let assembly_path = fixture_assembly_path();
    let output_path = std::env::temp_dir().join(format!(
        "rotappo-assembly-visualize-{}.dot",
        std::process::id()
    ));
    let output_path_str = output_path.to_string_lossy().to_string();

    let command = Commands::Assembly {
        assembly: AssemblyAction::Visualize {
            path: assembly_path.to_string_lossy().to_string(),
            view: "full".to_string(),
            format: "dot".to_string(),
            layout: "dot".to_string(),
            output: Some(output_path_str.clone()),
        },
    };

    let (path, view, format, layout, output) = match command {
        Commands::Assembly {
            assembly:
                AssemblyAction::Visualize {
                    path,
                    view,
                    format,
                    layout,
                    output,
                },
        } => (path, view, format, layout, output),
        _ => unreachable!("expected assembly visualize command"),
    };

    adapter_controller::assembly::visualize(adapter_controller::assembly::AssemblyVisualizeArgs {
        path,
        view,
        format,
        layout,
        output,
    })
    .await?;

    let dot = std::fs::read_to_string(&output_path)?;
    assert!(dot.contains("digraph"));
    Ok(())
}
