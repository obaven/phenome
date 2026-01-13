# Analytics Architecture

The analytics system runs as a dedicated service that collects cluster metrics,
stores them in SQLite (WAL mode), and serves gRPC queries to the TUI and ML
service. The TUI stays responsive by caching query results and degrading
gracefully when analytics is unavailable.

Key components:
- Analytics service (metrics collection, storage, aggregation, gRPC API)
- ML service (anomaly detection and recommendations)
- TUI analytics panels (real-time, historical, predictions, recommendations, insights)

Data flow:
1. ClusterManager polls metrics-server and emits MetricSample batches.
2. AnalyticsService writes raw samples and hourly aggregates.
3. AnalyticsService exposes metrics and aggregates via gRPC.
4. ML service consumes time series data and returns anomalies.
