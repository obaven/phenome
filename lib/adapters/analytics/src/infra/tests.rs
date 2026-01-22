use crate::cluster_manager::ClusterManager;

#[tokio::test]
async fn adds_and_lists_clusters() {
    let manager = ClusterManager::new();
    let id = manager.add_cluster("dev".to_string()).await.unwrap();
    let clusters = manager.list_clusters().await;
    assert_eq!(clusters.len(), 1);
    assert_eq!(clusters[0].id, id);
}
