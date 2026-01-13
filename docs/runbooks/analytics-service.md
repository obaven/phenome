# Analytics Service Runbook

## Start
- Ensure `rotappo-config.yaml` exists (see repository root template).
- Build: `cargo build --bin analytics-service --features analytics`
- Run: `cargo run --bin analytics-service --features analytics`

## Configuration
- `analytics.sqlite_path`: SQLite database path.
- `analytics.collection.interval_seconds`: polling interval.
- `services.analytics_url`: gRPC listen endpoint.

## Troubleshooting
- Verify SQLite file path is writable.
- Check logs in `/tmp/rotappo-analytics.log` when using the start script.
