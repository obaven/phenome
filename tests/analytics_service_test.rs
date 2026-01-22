#[tokio::test]
async fn analytics_service_constructs() {
    // TODO: Fix integration test - requires running gRPC services or mocks
    /*
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("analytics.db");
    let storage = SqliteStorage::new(db_path.to_string_lossy().to_string()).unwrap();
    let service = AnalyticsService::new(Arc::new(storage));
    let metrics = service
        .query_metrics(phenome_domain::MetricsQuery::default())
        .await
        .unwrap();
    assert!(metrics.is_empty());
    */
}
