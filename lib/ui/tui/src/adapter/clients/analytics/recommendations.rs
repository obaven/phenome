use anyhow::Result;

use phenome_adapter_analytics::grpc::analytics::{
    recommendation_action::Action as GrpcAction,
    recommendation_status::Status as GrpcStatus,
    GetRecommendationsRequest, Priority as GrpcPriority, RecommendationType as GrpcType,
};
use phenome_domain::{
    CostImpact, Priority, Recommendation, RecommendationAction, RecommendationStatus,
    RecommendationType, ResourceLimits,
};

use super::AnalyticsClient;

pub(super) async fn fetch_recommendations(client: &AnalyticsClient) -> Result<Vec<Recommendation>> {
    let mut grpc = client.client.clone();
    let request = GetRecommendationsRequest {
        limit: Some(20),
        ..Default::default()
    };
    let response = grpc.get_recommendations(request).await?;
    let recs = response.into_inner().recommendations;

    Ok(recs
        .into_iter()
        .map(|r| {
            let recommendation_type = map_type(r.recommendation_type());
            let priority = map_priority(r.priority());
            Recommendation {
                id: r.id,
                cluster_id: r.cluster_id,
                created_at: r.created_at,
                recommendation_type,
                priority,
            confidence: r.confidence,
            title: r.title,
            description: r.description,
            impact_estimate: r.impact_estimate,
            cost_impact: r.cost_impact.map(|cost| CostImpact {
                daily_change: cost.daily_change,
                currency: cost.currency,
            }),
            action: r
                .action
                .and_then(|a| a.action)
                .map(map_action)
                .unwrap_or(RecommendationAction::ScaleDeployment {
                    name: "unknown".into(),
                    from: 0,
                    to: 0,
                }),
            status: r
                .status
                .and_then(|s| s.status)
                .map(map_status)
                .unwrap_or(RecommendationStatus::Pending),
            }
        })
        .collect())
}

fn map_type(rec_type: GrpcType) -> RecommendationType {
    match rec_type {
        GrpcType::ScaleUp => RecommendationType::ScaleUp,
        GrpcType::ScaleDown => RecommendationType::ScaleDown,
        GrpcType::OptimizeResources => RecommendationType::OptimizeResources,
        GrpcType::AdjustLimits => RecommendationType::AdjustLimits,
        GrpcType::StorageOptimizations => RecommendationType::StorageOptimization,
        _ => RecommendationType::OptimizeResources,
    }
}

fn map_priority(priority: GrpcPriority) -> Priority {
    match priority {
        GrpcPriority::High => Priority::High,
        GrpcPriority::Medium => Priority::Medium,
        GrpcPriority::Low => Priority::Low,
        _ => Priority::Medium,
    }
}

fn map_action(action: GrpcAction) -> RecommendationAction {
    match action {
        GrpcAction::ScaleDeployment(s) => RecommendationAction::ScaleDeployment {
            name: s.name,
            from: s.from,
            to: s.to,
        },
        GrpcAction::UpdateLimits(u) => RecommendationAction::UpdateResourceLimits {
            resource: u.resource,
            limits: ResourceLimits {
                cpu: u.limits.as_ref().and_then(|l| l.cpu.clone()),
                memory: u.limits.as_ref().and_then(|l| l.memory.clone()),
            },
        },
        GrpcAction::ReclaimStorage(rs) => RecommendationAction::ReclaimStorage {
            volume: rs.volume,
            size_gb: rs.size_gb,
        },
    }
}

fn map_status(status: GrpcStatus) -> RecommendationStatus {
    match status {
        GrpcStatus::Pending(_) => RecommendationStatus::Pending,
        GrpcStatus::ScheduledAt(t) => RecommendationStatus::Scheduled { execute_at: t },
        GrpcStatus::AppliedAt(t) => RecommendationStatus::Applied { applied_at: t },
        GrpcStatus::DismissedReason(reason) => RecommendationStatus::Dismissed { reason },
    }
}
