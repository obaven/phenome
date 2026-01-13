use rotappo_domain::{MetricType, TimeSeries, TimeSeriesData, TimeSeriesPoint};

use crate::anomaly_detection::AnomalyDetector;

#[test]
fn detects_simple_anomaly() {
    let detector = AnomalyDetector::default();
    let series = TimeSeries {
        cluster_id: "cluster-1".to_string(),
        resource_id: "pod-a".to_string(),
        metric_type: MetricType::CpuUsage,
        unit: "cores".to_string(),
        points: vec![
            TimeSeriesPoint {
                timestamp: 1,
                value: 1.0,
            },
            TimeSeriesPoint {
                timestamp: 2,
                value: 1.1,
            },
            TimeSeriesPoint {
                timestamp: 3,
                value: 10.0,
            },
        ],
    };
    let data = TimeSeriesData {
        cluster_id: "cluster-1".to_string(),
        range: rotappo_domain::TimeRange { start_ms: 0, end_ms: 3 },
        series: vec![series],
    };

    let anomalies = detector.detect(&data).unwrap();
    assert!(!anomalies.is_empty());
}
