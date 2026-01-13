# gRPC API

The analytics and ML services expose gRPC endpoints for:
- Metrics query and aggregation
- Time series retrieval
- Anomaly detection results
- Recommendation lists

Endpoints are configurable via `rotappo-config.yaml` and default to:
- Analytics: http://localhost:50051
- ML: http://localhost:50052
