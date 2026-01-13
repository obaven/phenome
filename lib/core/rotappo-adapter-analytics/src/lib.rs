//! Analytics adapter implementations.

pub mod aggregator;
pub mod analytics_engine;
pub mod analytics_service;
pub mod cache;
pub mod circuit_breaker;
pub mod cluster_manager;
pub mod grpc;
pub mod metrics_collector;
pub mod notification;
pub mod scheduler;
pub mod storage;

pub use analytics_service::AnalyticsService;
pub use cluster_manager::ClusterManager;

#[cfg(test)]
mod cluster_manager_test;
