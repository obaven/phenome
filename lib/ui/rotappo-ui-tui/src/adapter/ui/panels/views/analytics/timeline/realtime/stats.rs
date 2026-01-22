use phenome_domain::{MetricSample, MetricType, ResourceType};

pub(super) struct MetricTotals {
    pub(super) cpu_sum: f64,
    pub(super) mem_sum: f64,
    pub(super) cpu_valid: bool,
    pub(super) mem_valid: bool,
}

pub(super) fn aggregate_metrics(metrics: &[MetricSample]) -> MetricTotals {
    let mut cpu_sum = 0.0;
    let mut mem_sum = 0.0;
    let mut cpu_count = 0;
    let mut mem_count = 0;

    for sample in metrics {
        match sample.metric_type {
            MetricType::CpuUsage => {
                cpu_sum += sample.value;
                cpu_count += 1;
            }
            MetricType::MemoryUsage => {
                mem_sum += sample.value;
                mem_count += 1;
            }
            _ => {}
        }
    }

    MetricTotals {
        cpu_sum,
        mem_sum,
        cpu_valid: cpu_count > 0,
        mem_valid: mem_count > 0,
    }
}

pub(super) fn build_info(metrics: &[MetricSample]) -> String {
    let pod_count = metrics
        .iter()
        .filter(|s| matches!(s.resource_type, ResourceType::Pod))
        .count()
        / 2;
    let node_count = metrics
        .iter()
        .filter(|s| matches!(s.resource_type, ResourceType::Node))
        .count()
        / 2;

    format!(
        "Samples: {} | Pods: {} | Nodes: {}",
        metrics.len(),
        pod_count,
        node_count
    )
}
