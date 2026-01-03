use anyhow::{anyhow, Result};
use std::env;
use std::path::PathBuf;

use rotappo::adapters::bootstrappo::BootstrappoBackend;
use rotappo::cli::{format_actions, format_events, format_snapshot, OutputMode};
use rotappo::logging::LogStreamConfig;

fn main() -> Result<()> {
    let args = env::args().skip(1).collect::<Vec<_>>();
    if args.iter().any(|arg| arg == "--help" || arg == "-h") {
        print_help();
        return Ok(());
    }

    let mut output = OutputMode::Plain;
    let mut command = "snapshot".to_string();
    let mut config_path: Option<PathBuf> = None;
    let mut plan_path: Option<PathBuf> = None;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "actions" | "snapshot" | "logs" => {
                command = args[i].clone();
            }
            "--output" | "-o" => {
                let value = args.get(i + 1).ok_or_else(|| anyhow!("Missing output mode"))?;
                output = OutputMode::from_str(value)
                    .ok_or_else(|| anyhow!("Invalid output mode: {}", value))?;
                i += 1;
            }
            "--config" => {
                let value = args.get(i + 1).ok_or_else(|| anyhow!("Missing config path"))?;
                config_path = Some(PathBuf::from(value));
                i += 1;
            }
            "--plan" => {
                let value = args.get(i + 1).ok_or_else(|| anyhow!("Missing plan path"))?;
                plan_path = Some(PathBuf::from(value));
                i += 1;
            }
            _ => {}
        }
        i += 1;
    }

    let backend = BootstrappoBackend::from_paths(config_path, plan_path)?;
    let runtime = backend.runtime();

    let log_config = LogStreamConfig::default();
    let output_text = match command.as_str() {
        "actions" => format_actions(output, runtime.registry().actions())?,
        "snapshot" => format_snapshot(output, runtime.snapshot())?,
        "logs" => {
            let events = runtime
                .events()
                .iter()
                .filter(|event| log_config.filter.matches(event.level))
                .cloned()
                .collect::<Vec<_>>();
            format_events(output, &events)?
        }
        other => return Err(anyhow!("Unknown command: {}", other)),
    };

    println!("{}", output_text);
    Ok(())
}

fn print_help() {
    println!(
        "rotappo [actions|snapshot|logs] [--output plain|json|ndjson] [--config PATH] [--plan PATH]"
    );
}
