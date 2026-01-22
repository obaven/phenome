use std::env;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};

use phenome_adapter_ml::grpc::GrpcServer;
use phenome_adapter_ml::MlService;
use phenome_domain::PhenomeConfig;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config_path = config_path();
    let config = PhenomeConfig::load_from_path(&config_path)?;
    let _service = MlService::new();

    let addr = parse_addr(&config.services.ml_url)
        .unwrap_or_else(|| "127.0.0.1:50052".parse().expect("invalid fallback addr"));
    GrpcServer::serve(addr).await?;
    Ok(())
}

fn config_path() -> PathBuf {
    if let Ok(path) = env::var("PHENOME_CONFIG_PATH") {
        return PathBuf::from(path);
    }

    if let Ok(home) = env::var("HOME") {
        return Path::new(&home).join(".phenome").join("config.yaml");
    }

    PathBuf::from("phenome-config.yaml")
}

fn parse_addr(raw: &str) -> Option<SocketAddr> {
    let trimmed = raw.trim();
    let value = trimmed
        .strip_prefix("http://")
        .or_else(|| trimmed.strip_prefix("https://"))
        .unwrap_or(trimmed);
    value.parse().ok()
}
