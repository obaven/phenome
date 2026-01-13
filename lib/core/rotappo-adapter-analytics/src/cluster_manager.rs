use anyhow::Result;
use chrono::Utc;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use rotappo_domain::{ClusterHealth, ClusterId, ClusterMetadata, MetricSample, MetricsQuery};

#[derive(Clone, Default)]
pub struct ClusterManager {
    clusters: Arc<RwLock<HashMap<ClusterId, ClusterMetadata>>>,
    clients: Arc<RwLock<HashMap<ClusterId, kube::Client>>>,
}

impl std::fmt::Debug for ClusterManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let clusters_len = self.clusters.try_read().map(|g| g.len()).ok();
        let clients_len = self.clients.try_read().map(|g| g.len()).ok();
        f.debug_struct("ClusterManager")
            .field("clusters_count", &clusters_len)
            .field("clients_count", &clients_len)
            .finish()
    }
}

impl ClusterManager {
    pub fn new() -> Self {
        Self {
            clusters: Arc::new(RwLock::new(HashMap::new())),
            clients: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn add_cluster(&self, context: String) -> Result<ClusterId> {
        let mut clusters = self.clusters.write().await;
        let id = context.clone();
        let metadata = ClusterMetadata {
            id: id.clone(),
            name: context.clone(),
            context,
            api_server: String::new(),
            health_status: ClusterHealth::Healthy,
            last_seen: Utc::now().timestamp_millis(),
            pod_count: 0,
            node_count: 0,
            namespace_count: 0,
        };
        clusters.insert(id.clone(), metadata);
        Ok(id)
    }

    pub async fn remove_cluster(&self, id: &ClusterId) -> Result<()> {
        let mut clusters = self.clusters.write().await;
        clusters.remove(id);
        Ok(())
    }

    pub async fn list_clusters(&self) -> Vec<ClusterMetadata> {
        let clusters = self.clusters.read().await;
        clusters.values().cloned().collect()
    }

    pub async fn get_cluster_health(&self, id: &ClusterId) -> ClusterHealth {
        let clusters = self.clusters.read().await;
        clusters
            .get(id)
            .map(|cluster| cluster.health_status)
            .unwrap_or(ClusterHealth::Unreachable)
    }

    async fn get_client(&self, context: &str) -> Result<kube::Client> {
        let clients = self.clients.read().await;
        if let Some(client) = clients.get(context) {
            return Ok(client.clone());
        }
        drop(clients);

        let mut clients = self.clients.write().await;
        // Double-check
        if let Some(client) = clients.get(context) {
            return Ok(client.clone());
        }

        // In a real scenario, we'd load specific kubeconfig for the context
        // For now, we try to infer from default or assume context switching
        // This is a simplification; handling multi-cluster kubeconfigs properly is complex
        let options = kube::config::KubeConfigOptions {
            context: Some(context.to_string()),
            ..Default::default()
        };
        let user_config = kube::config::Kubeconfig::read()
            .map_err(|e| anyhow::anyhow!("Failed into read kubeconfig: {}", e))?;
        let config = kube::Config::from_custom_kubeconfig(user_config, &options).await?;
        let client = kube::Client::try_from(config)?;

        clients.insert(context.to_string(), client.clone());
        Ok(client)
    }

    pub async fn query_metrics(
        &self,
        cluster_id: &ClusterId,
        query: MetricsQuery,
    ) -> Result<Vec<MetricSample>> {
        let client = match self.get_client(cluster_id).await {
            Ok(c) => c,
            Err(e) => {
                tracing::warn!("Failed to get client for cluster {}: {}", cluster_id, e);
                return Ok(Vec::new()); // Fallback to empty
            }
        };

        let mut samples = Vec::new();

        // Fetch Node Metrics
        if query.resource_type.is_none()
            || query.resource_type == Some(rotappo_domain::ResourceType::Node)
        {
            if let Ok(node_metrics) = self.fetch_node_metrics(&client, cluster_id).await {
                samples.extend(node_metrics);
            }
        }

        // Fetch Pod Metrics
        if query.resource_type.is_none()
            || query.resource_type == Some(rotappo_domain::ResourceType::Pod)
        {
            if let Ok(pod_metrics) = self.fetch_pod_metrics(&client, cluster_id).await {
                samples.extend(pod_metrics);
            }
        }

        Ok(samples)
    }

    async fn fetch_node_metrics(
        &self,
        client: &kube::Client,
        cluster_id: &str,
    ) -> Result<Vec<MetricSample>> {
        let gvk = kube::api::GroupVersionKind::gvk("metrics.k8s.io", "v1beta1", "NodeMetrics");
        let api_resource = kube::api::ApiResource::from_gvk(&gvk);
        let metrics_api =
            kube::Api::<kube::api::DynamicObject>::all_with(client.clone(), &api_resource);

        let node_metrics = match metrics_api.list(&kube::api::ListParams::default()).await {
            Ok(list) => list,
            Err(e) => {
                tracing::warn!(
                    "Failed to fetch node metrics (is metrics-server installed?): {}",
                    e
                );
                return Ok(Vec::new());
            }
        };

        let mut samples = Vec::new();
        for metric in node_metrics {
            let name = metric.metadata.name.unwrap_or_default();
            // Unpack usage
            if let Some(usage) = metric.data.get("usage").and_then(|u| u.as_object()) {
                if let Some(cpu) = usage.get("cpu").and_then(|v| v.as_str()) {
                    // parse cpu (e.g. "123n" or "0.1")
                    let val = parse_k8s_quantity(cpu);
                    samples.push(MetricSample {
                        cluster_id: cluster_id.to_string(),
                        resource_type: rotappo_domain::ResourceType::Node,
                        resource_id: name.clone(),
                        metric_type: rotappo_domain::MetricType::CpuUsage,
                        timestamp: Utc::now().timestamp_millis(),
                        value: val,
                        unit: "cores".to_string(),
                    });
                }
                if let Some(mem) = usage.get("memory").and_then(|v| v.as_str()) {
                    let val = parse_k8s_quantity(mem);
                    samples.push(MetricSample {
                        cluster_id: cluster_id.to_string(),
                        resource_type: rotappo_domain::ResourceType::Node,
                        resource_id: name.clone(),
                        metric_type: rotappo_domain::MetricType::MemoryUsage,
                        timestamp: Utc::now().timestamp_millis(),
                        value: val,
                        unit: "bytes".to_string(),
                    });
                }
            }
        }
        Ok(samples)
    }

    async fn fetch_pod_metrics(
        &self,
        client: &kube::Client,
        cluster_id: &str,
    ) -> Result<Vec<MetricSample>> {
        let gvk = kube::api::GroupVersionKind::gvk("metrics.k8s.io", "v1beta1", "PodMetrics");
        let api_resource = kube::api::ApiResource::from_gvk(&gvk);
        let metrics_api =
            kube::Api::<kube::api::DynamicObject>::all_with(client.clone(), &api_resource);

        let pod_metrics = match metrics_api.list(&kube::api::ListParams::default()).await {
            Ok(list) => list,
            Err(e) => {
                tracing::warn!("Failed to fetch pod metrics: {}", e);
                return Ok(Vec::new());
            }
        };

        let mut samples = Vec::new();
        for metric in pod_metrics {
            let name = metric.metadata.name.unwrap_or_default();
            let namespace = metric.metadata.namespace.unwrap_or_default();
            let resource_id = format!("{}/{}", namespace, name);

            // Pod metrics have containers list
            if let Some(containers) = metric.data.get("containers").and_then(|c| c.as_array()) {
                let mut total_cpu = 0.0;
                let mut total_mem = 0.0;

                for c in containers {
                    if let Some(usage) = c.get("usage").and_then(|u| u.as_object()) {
                        if let Some(cpu) = usage.get("cpu").and_then(|v| v.as_str()) {
                            total_cpu += parse_k8s_quantity(cpu);
                        }
                        if let Some(mem) = usage.get("memory").and_then(|v| v.as_str()) {
                            total_mem += parse_k8s_quantity(mem);
                        }
                    }
                }

                samples.push(MetricSample {
                    cluster_id: cluster_id.to_string(),
                    resource_type: rotappo_domain::ResourceType::Pod,
                    resource_id: resource_id.clone(),
                    metric_type: rotappo_domain::MetricType::CpuUsage,
                    timestamp: Utc::now().timestamp_millis(),
                    value: total_cpu,
                    unit: "cores".to_string(),
                });
                samples.push(MetricSample {
                    cluster_id: cluster_id.to_string(),
                    resource_type: rotappo_domain::ResourceType::Pod,
                    resource_id: resource_id.clone(),
                    metric_type: rotappo_domain::MetricType::MemoryUsage,
                    timestamp: Utc::now().timestamp_millis(),
                    value: total_mem,
                    unit: "bytes".to_string(),
                });
            }
        }
        Ok(samples)
    }

    // Helper for parsing k8s quantities
    // fn parse_k8s_quantity... needs to be added or used if available

    pub async fn query_all_clusters(
        &self,
        query: MetricsQuery,
    ) -> Vec<(ClusterId, Result<Vec<MetricSample>>)> {
        let clusters = self.list_clusters().await;
        let mut set = tokio::task::JoinSet::new();

        for cluster in clusters {
            let manager = self.clone();
            let q = query.clone();
            let c_id = cluster.id.clone();
            set.spawn(async move {
                let res = manager.query_metrics(&c_id, q).await;
                (c_id, res)
            });
        }

        let mut results = Vec::new();
        while let Some(res) = set.join_next().await {
            match res {
                Ok(val) => results.push(val),
                Err(e) => tracing::error!("Task join error: {}", e),
            }
        }
        results
    }
}

fn parse_k8s_quantity(q: &str) -> f64 {
    let q = q.trim();
    if let Ok(val) = q.parse::<f64>() {
        return val;
    }

    // Simple parsing for m, Ki, Mi, Gi, n
    if let Some(stripped) = q.strip_suffix('n') {
        return stripped.parse::<f64>().unwrap_or(0.0) / 1_000_000_000.0;
    }
    if let Some(stripped) = q.strip_suffix('u') {
        return stripped.parse::<f64>().unwrap_or(0.0) / 1_000_000.0;
    }
    if let Some(stripped) = q.strip_suffix('m') {
        return stripped.parse::<f64>().unwrap_or(0.0) / 1000.0;
    }
    if let Some(stripped) = q.strip_suffix("Ki") {
        return stripped.parse::<f64>().unwrap_or(0.0) * 1024.0;
    }
    if let Some(stripped) = q.strip_suffix("Mi") {
        return stripped.parse::<f64>().unwrap_or(0.0) * 1024.0 * 1024.0;
    }
    if let Some(stripped) = q.strip_suffix("Gi") {
        return stripped.parse::<f64>().unwrap_or(0.0) * 1024.0 * 1024.0 * 1024.0;
    }

    // Fallback logic could get fancy but strictly implementation of "usage" is usually simple
    0.0
}
