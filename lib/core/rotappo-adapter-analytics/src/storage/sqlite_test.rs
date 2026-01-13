use rotappo_domain::{MetricSample, MetricType, MetricsQuery, ResourceType};

use crate::storage::port::StoragePort;
use crate::storage::sqlite::SqliteStorage;

#[tokio::test]
async fn sqlite_inserts_and_queries_metrics() {
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("analytics.db");
    let storage = SqliteStorage::new(db_path.to_string_lossy().to_string()).unwrap();

    storage
        .insert_metrics(vec![MetricSample {
            cluster_id: "cluster-1".to_string(),
            resource_type: ResourceType::Pod,
            resource_id: "pod-a".to_string(),
            metric_type: MetricType::CpuUsage,
            timestamp: 1_000,
            value: 0.42,
            unit: "cores".to_string(),
        }])
        .await
        .unwrap();

    let results = storage
        .query_metrics(MetricsQuery {
            cluster_id: Some("cluster-1".to_string()),
            resource_type: Some(ResourceType::Pod),
            resource_ids: vec!["pod-a".to_string()],
            metric_types: vec![MetricType::CpuUsage],
            time_range: None,
        })
        .await
        .unwrap();

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].resource_id, "pod-a");
}
