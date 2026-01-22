use anyhow::{Context, Result};
use async_trait::async_trait;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::{Connection, params};
use serde::{Serialize, de::DeserializeOwned};
use std::time::Duration;

use phenome_domain::{AggregatedMetric, AggregatedQuery, MetricSample, MetricsQuery, TimeRange};

use super::port::StoragePort;

const SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS metrics_raw (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    cluster_id TEXT NOT NULL,
    resource_type TEXT NOT NULL,
    resource_id TEXT NOT NULL,
    metric_type TEXT NOT NULL,
    timestamp INTEGER NOT NULL,
    value REAL NOT NULL,
    unit TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_metrics_raw_cluster_time ON metrics_raw (cluster_id, timestamp);
CREATE INDEX IF NOT EXISTS idx_metrics_raw_resource_time ON metrics_raw (resource_id, timestamp);

CREATE TABLE IF NOT EXISTS metrics_aggregated (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    cluster_id TEXT NOT NULL,
    resource_type TEXT NOT NULL,
    metric_type TEXT NOT NULL,
    window_start INTEGER NOT NULL,
    window_duration INTEGER NOT NULL,
    count INTEGER NOT NULL,
    sum REAL NOT NULL,
    min REAL NOT NULL,
    max REAL NOT NULL,
    avg REAL NOT NULL,
    p50 REAL NOT NULL,
    p95 REAL NOT NULL,
    p99 REAL NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_metrics_agg_cluster_window
    ON metrics_aggregated (cluster_id, window_start);

CREATE TABLE IF NOT EXISTS anomalies (
    id TEXT PRIMARY KEY,
    cluster_id TEXT NOT NULL,
    resource_id TEXT NOT NULL,
    detected_at INTEGER NOT NULL,
    metric_type TEXT NOT NULL,
    severity TEXT NOT NULL,
    confidence REAL NOT NULL,
    description TEXT NOT NULL,
    baseline_value REAL NOT NULL,
    observed_value REAL NOT NULL,
    deviation_sigma REAL NOT NULL,
    related_metrics TEXT,
    root_cause TEXT
);
CREATE INDEX IF NOT EXISTS idx_anomalies_cluster_time
    ON anomalies (cluster_id, detected_at);

CREATE TABLE IF NOT EXISTS recommendations (
    id TEXT PRIMARY KEY,
    cluster_id TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    recommendation_type TEXT NOT NULL,
    priority TEXT NOT NULL,
    confidence REAL NOT NULL,
    title TEXT NOT NULL,
    description TEXT NOT NULL,
    impact_estimate TEXT NOT NULL,
    cost_impact_daily REAL,
    cost_impact_currency TEXT,
    action TEXT NOT NULL,
    status TEXT NOT NULL,
    status_data TEXT
);
CREATE INDEX IF NOT EXISTS idx_recommendations_cluster_status
    ON recommendations (cluster_id, status);

CREATE TABLE IF NOT EXISTS clusters (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    context TEXT NOT NULL UNIQUE,
    api_server TEXT NOT NULL,
    health_status TEXT NOT NULL,
    last_seen INTEGER NOT NULL,
    pod_count INTEGER NOT NULL,
    node_count INTEGER NOT NULL,
    namespace_count INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS scheduled_actions (
    id TEXT PRIMARY KEY,
    execute_at INTEGER NOT NULL,
    recommendation_id TEXT NOT NULL,
    action TEXT NOT NULL,
    status TEXT NOT NULL,
    status_data TEXT
);
CREATE INDEX IF NOT EXISTS idx_scheduled_actions_execute_at
    ON scheduled_actions (execute_at);
"#;

#[derive(Debug, Clone)]
pub struct RetentionConfig {
    pub raw_days: i64,
    pub aggregated_days: i64,
}

impl Default for RetentionConfig {
    fn default() -> Self {
        Self {
            raw_days: 7,
            aggregated_days: 30,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SqliteStorage {
    pool: Pool<SqliteConnectionManager>,
    retention: RetentionConfig,
}

impl SqliteStorage {
    pub fn new(path: impl Into<String>) -> Result<Self> {
        Self::with_retention(path, RetentionConfig::default())
    }

    pub fn with_retention(path: impl Into<String>, retention: RetentionConfig) -> Result<Self> {
        let manager = SqliteConnectionManager::file(path.into());
        let pool = Pool::builder()
            .max_size(10)
            .build(manager)
            .context("failed to create sqlite pool")?;
        let storage = Self { pool, retention };
        storage.init()?;
        Ok(storage)
    }

    pub fn run_retention_cleanup(&self, now_ms: i64) -> Result<()> {
        let conn = self.pool.get().context("failed to get sqlite connection")?;
        let raw_cutoff = now_ms - self.retention.raw_days * 24 * 60 * 60 * 1000;
        let agg_cutoff = now_ms - self.retention.aggregated_days * 24 * 60 * 60 * 1000;
        conn.execute(
            "DELETE FROM metrics_raw WHERE timestamp < ?1",
            params![raw_cutoff],
        )?;
        conn.execute(
            "DELETE FROM metrics_aggregated WHERE window_start < ?1",
            params![agg_cutoff],
        )?;
        Ok(())
    }

    fn init(&self) -> Result<()> {
        let conn = self.pool.get().context("failed to get sqlite connection")?;
        configure_sqlite(&conn)?;
        conn.execute_batch(SCHEMA)
            .context("failed to apply sqlite schema")?;
        Ok(())
    }
}

#[async_trait]
impl StoragePort for SqliteStorage {
    async fn insert_metrics(&self, samples: Vec<MetricSample>) -> Result<()> {
        if samples.is_empty() {
            return Ok(());
        }

        let mut conn = self.pool.get().context("failed to get sqlite connection")?;
        let mut offset = 0;
        while offset < samples.len() {
            let end = (offset + 1000).min(samples.len());
            let tx = conn.transaction().context("failed to begin transaction")?;
            {
                let mut stmt = tx.prepare(
                    "INSERT INTO metrics_raw (cluster_id, resource_type, resource_id, metric_type, timestamp, value, unit)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                )?;
                for sample in &samples[offset..end] {
                    stmt.execute(params![
                        sample.cluster_id,
                        encode_enum(&sample.resource_type)?,
                        sample.resource_id,
                        encode_enum(&sample.metric_type)?,
                        sample.timestamp,
                        sample.value,
                        sample.unit
                    ])?;
                }
            }
            tx.commit().context("failed to commit metrics batch")?;
            offset = end;
        }

        Ok(())
    }

    async fn query_metrics(&self, query: MetricsQuery) -> Result<Vec<MetricSample>> {
        let conn = self.pool.get().context("failed to get sqlite connection")?;
        let mut stmt = conn.prepare(
            "SELECT cluster_id, resource_type, resource_id, metric_type, timestamp, value, unit
             FROM metrics_raw",
        )?;
        let rows = stmt.query_map([], |row| {
            let resource_type_str: String = row.get(1)?;
            let metric_type_str: String = row.get(3)?;

            Ok(MetricSample {
                cluster_id: row.get(0)?,
                resource_type: decode_enum(&resource_type_str)
                    .map_err(|e| rusqlite::Error::ToSqlConversionFailure(e.into()))?,
                resource_id: row.get(2)?,
                metric_type: decode_enum(&metric_type_str)
                    .map_err(|e| rusqlite::Error::ToSqlConversionFailure(e.into()))?,
                timestamp: row.get(4)?,
                value: row.get(5)?,
                unit: row.get(6)?,
            })
        })?;

        let mut samples = Vec::new();
        for row in rows {
            samples.push(row?);
        }

        Ok(filter_metrics(samples, &query))
    }

    async fn insert_aggregated(&self, metrics: Vec<AggregatedMetric>) -> Result<()> {
        if metrics.is_empty() {
            return Ok(());
        }

        let mut conn = self.pool.get().context("failed to get sqlite connection")?;
        let tx = conn.transaction().context("failed to begin transaction")?;
        {
            let mut stmt = tx.prepare(
                "INSERT INTO metrics_aggregated
                (cluster_id, resource_type, metric_type, window_start, window_duration, count, sum, min, max, avg, p50, p95, p99)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
            )?;
            for metric in metrics {
                stmt.execute(params![
                    metric.cluster_id,
                    encode_enum(&metric.resource_type)?,
                    encode_enum(&metric.metric_type)?,
                    metric.window_start,
                    metric.window_duration.as_millis() as i64,
                    metric.count as i64,
                    metric.sum,
                    metric.min,
                    metric.max,
                    metric.avg,
                    metric.p50,
                    metric.p95,
                    metric.p99
                ])?;
            }
        }
        tx.commit().context("failed to commit aggregated metrics")?;
        Ok(())
    }

    async fn query_aggregated(&self, query: AggregatedQuery) -> Result<Vec<AggregatedMetric>> {
        let conn = self.pool.get().context("failed to get sqlite connection")?;
        let mut stmt = conn.prepare(
            "SELECT cluster_id, resource_type, metric_type, window_start, window_duration, count, sum, min, max, avg, p50, p95, p99
             FROM metrics_aggregated",
        )?;
        let rows = stmt.query_map([], |row| {
            let duration_ms: i64 = row.get(4)?;
            let resource_type_str: String = row.get(1)?;
            let metric_type_str: String = row.get(2)?;

            Ok(AggregatedMetric {
                cluster_id: row.get(0)?,
                resource_type: decode_enum(&resource_type_str)
                    .map_err(|e| rusqlite::Error::ToSqlConversionFailure(e.into()))?,
                metric_type: decode_enum(&metric_type_str)
                    .map_err(|e| rusqlite::Error::ToSqlConversionFailure(e.into()))?,
                window_start: row.get(3)?,
                window_duration: Duration::from_millis(duration_ms.max(0) as u64),
                count: row.get::<_, i64>(5)? as u64,
                sum: row.get(6)?,
                min: row.get(7)?,
                max: row.get(8)?,
                avg: row.get(9)?,
                p50: row.get(10)?,
                p95: row.get(11)?,
                p99: row.get(12)?,
            })
        })?;

        let mut metrics = Vec::new();
        for row in rows {
            metrics.push(row?);
        }

        Ok(filter_aggregated(metrics, &query))
    }

    async fn insert_anomalies(&self, anomalies: Vec<phenome_domain::Anomaly>) -> Result<()> {
        if anomalies.is_empty() {
            return Ok(());
        }

        let mut conn = self.pool.get().context("failed to get sqlite connection")?;
        let tx = conn.transaction().context("failed to begin transaction")?;
        {
            let mut stmt = tx.prepare(
                "INSERT OR REPLACE INTO anomalies 
                 (id, cluster_id, resource_id, detected_at, metric_type, severity, confidence, description, baseline_value, observed_value, deviation_sigma, related_metrics, root_cause)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
            )?;
            for anomaly in anomalies {
                stmt.execute(params![
                    anomaly.id,
                    anomaly.cluster_id,
                    anomaly.resource_id,
                    anomaly.detected_at,
                    encode_enum(&anomaly.metric_type)?,
                    encode_enum(&anomaly.severity)?,
                    anomaly.confidence,
                    anomaly.description,
                    anomaly.baseline_value,
                    anomaly.observed_value,
                    anomaly.deviation_sigma,
                    serde_json::to_string(&anomaly.related_metrics).unwrap_or_default(),
                    anomaly.root_cause
                ])?;
            }
        }
        tx.commit().context("failed to commit anomalies")?;
        Ok(())
    }

    async fn cleanup_retention(&self) -> Result<()> {
        self.run_retention_cleanup(chrono::Utc::now().timestamp_millis())
    }

    async fn insert_schedule(&self, action: phenome_domain::ScheduledAction) -> Result<()> {
        let conn = self.pool.get().context("failed to get sqlite connection")?;
        conn.execute(
            "INSERT INTO scheduled_actions (id, execute_at, recommendation_id, action, status)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                action.id,
                action.execute_at,
                action.recommendation_id,
                serde_json::to_string(&action.action)?,
                serde_json::to_string(&action.status)?,
            ],
        )?;
        Ok(())
    }

    async fn update_schedule(&self, action: phenome_domain::ScheduledAction) -> Result<()> {
        let conn = self.pool.get().context("failed to get sqlite connection")?;
        conn.execute(
            "UPDATE scheduled_actions SET execute_at = ?2, recommendation_id = ?3, action = ?4, status = ?5
             WHERE id = ?1",
            params![
                action.id,
                action.execute_at,
                action.recommendation_id,
                serde_json::to_string(&action.action)?,
                serde_json::to_string(&action.status)?,
            ],
        )?;
        Ok(())
    }

    async fn get_all_schedules(&self) -> Result<Vec<phenome_domain::ScheduledAction>> {
        let conn = self.pool.get().context("failed to get sqlite connection")?;
        let mut stmt = conn.prepare(
            "SELECT id, execute_at, recommendation_id, action, status FROM scheduled_actions",
        )?;
        let rows = stmt.query_map([], |row| {
            let action_str: String = row.get(3)?;
            let status_str: String = row.get(4)?;
            Ok(phenome_domain::ScheduledAction {
                id: row.get(0)?,
                execute_at: row.get(1)?,
                recommendation_id: row.get(2)?,
                action: serde_json::from_str(&action_str)
                    .map_err(|e| rusqlite::Error::ToSqlConversionFailure(e.into()))?,
                status: serde_json::from_str(&status_str)
                    .map_err(|e| rusqlite::Error::ToSqlConversionFailure(e.into()))?,
            })
        })?;

        let mut actions = Vec::new();
        for row in rows {
            actions.push(row?);
        }
        Ok(actions)
    }
}

fn configure_sqlite(conn: &Connection) -> Result<()> {
    conn.pragma_update(None, "journal_mode", &"WAL")?;
    conn.pragma_update(None, "synchronous", &"NORMAL")?;
    conn.pragma_update(None, "cache_size", &64_000)?;
    Ok(())
}

fn encode_enum<T: Serialize>(value: &T) -> Result<String> {
    let json = serde_json::to_value(value)?;
    match json {
        serde_json::Value::String(value) => Ok(value),
        _ => anyhow::bail!("expected string enum encoding"),
    }
}

fn decode_enum<T: DeserializeOwned>(value: &str) -> Result<T> {
    let value = serde_json::Value::String(value.to_string());
    Ok(serde_json::from_value(value)?)
}

fn filter_metrics(mut samples: Vec<MetricSample>, query: &MetricsQuery) -> Vec<MetricSample> {
    samples.retain(|sample| matches_metrics_query(sample, query));
    samples
}

fn matches_metrics_query(sample: &MetricSample, query: &MetricsQuery) -> bool {
    query
        .cluster_id
        .as_ref()
        .map_or(true, |id| id == &sample.cluster_id)
        && query
            .resource_type
            .as_ref()
            .map_or(true, |resource_type| resource_type == &sample.resource_type)
        && (query.resource_ids.is_empty()
            || query
                .resource_ids
                .iter()
                .any(|id| id == &sample.resource_id))
        && (query.metric_types.is_empty()
            || query
                .metric_types
                .iter()
                .any(|metric_type| metric_type == &sample.metric_type))
        && query
            .time_range
            .as_ref()
            .map_or(true, |range| timestamp_in_range(sample.timestamp, range))
}

fn filter_aggregated(
    mut metrics: Vec<AggregatedMetric>,
    query: &AggregatedQuery,
) -> Vec<AggregatedMetric> {
    metrics.retain(|metric| {
        query
            .cluster_id
            .as_ref()
            .map_or(true, |id| id == &metric.cluster_id)
            && query
                .resource_type
                .as_ref()
                .map_or(true, |resource_type| resource_type == &metric.resource_type)
            && (query.metric_types.is_empty()
                || query
                    .metric_types
                    .iter()
                    .any(|metric_type| metric_type == &metric.metric_type))
            && query
                .time_range
                .as_ref()
                .map_or(true, |range| timestamp_in_range(metric.window_start, range))
    });
    metrics
}

fn timestamp_in_range(timestamp: i64, range: &TimeRange) -> bool {
    if range.end_ms < range.start_ms {
        return false;
    }
    timestamp >= range.start_ms && timestamp <= range.end_ms
}
