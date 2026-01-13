# Troubleshooting

## Analytics panels show "Metrics unavailable"
- Confirm `metrics-server` is installed in the cluster.
- Check analytics service logs for collection errors.
- Verify `ROTAPPO_ANALYTICS_URL` points to the running service.

## ML recommendations are empty
- Ensure the ML service is running and reachable.
- Confirm analytics has stored metrics for the selected time window.

## Services fail to start
- Validate `rotappo-config.yaml` paths and permissions.
- Ensure required ports (50051/50052) are available.
