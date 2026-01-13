# ML Service Runbook

## Start
- Build: `cargo build --bin ml-service --features ml`
- Run: `cargo run --bin ml-service --features ml`

## Configuration
- `ml.models`: model selection.
- `services.ml_url`: gRPC listen endpoint.

## Troubleshooting
- Ensure analytics service is reachable for historical data.
- Check logs in `/tmp/rotappo-ml.log` when using the start script.
