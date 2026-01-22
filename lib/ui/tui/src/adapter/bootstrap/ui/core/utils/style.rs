use ratatui::style::{Color, Style};
use ratatui::text::Line;

use primer::application::flows::reconcile::visualize::LayerType;
use phenome_ports::{ComponentState, ComponentStatus};

pub fn style_line(line: String, selected: bool) -> Line<'static> {
    if selected {
        Line::styled(line, Style::default().bg(Color::Blue).fg(Color::White))
    } else {
        Line::from(line)
    }
}

pub fn status_icon(status: ComponentStatus) -> &'static str {
    match status {
        ComponentStatus::Pending => "PEND",
        ComponentStatus::Running => "RUN",
        ComponentStatus::Complete => "OK",
        ComponentStatus::Failed => "FAIL",
        ComponentStatus::Deferred => "DEF",
    }
}

pub fn layer_label(layer: LayerType) -> &'static str {
    match layer {
        LayerType::Network => "Network & Connectivity",
        LayerType::Storage => "Storage",
        LayerType::Security => "Security",
        LayerType::System => "System",
        LayerType::Datastores => "Datastores",
        LayerType::Observability => "Observability",
        LayerType::Analytics => "Analytics",
        LayerType::Entertainment => "Entertainment",
        LayerType::Infrastructure => "Infrastructure",
        LayerType::GitOps => "GitOps",
        LayerType::Unknown => "Other",
    }
}

pub fn layer_from_domain(domain: &str) -> LayerType {
    match domain.to_lowercase().as_str() {
        "network" => LayerType::Network,
        "storage" => LayerType::Storage,
        "security" => LayerType::Security,
        "system" => LayerType::System,
        "datastores" | "datastore" | "database" => LayerType::Datastores,
        "observability" => LayerType::Observability,
        "analytics" => LayerType::Analytics,
        "entertainment" | "productivity" => LayerType::Entertainment,
        "infrastructure" => LayerType::Infrastructure,
        "gitops" => LayerType::GitOps,
        _ => LayerType::Unknown,
    }
}

pub fn format_status(state: &ComponentState) -> String {
    match state.status {
        ComponentStatus::Pending => "PEND Pending".to_string(),
        ComponentStatus::Running => {
            let phase = state
                .readiness
                .as_ref()
                .map(|r| r.basic.summary.clone())
                .unwrap_or_else(|| "Running".to_string());
            format!("RUN {phase}")
        }
        ComponentStatus::Complete => "OK Complete".to_string(),
        ComponentStatus::Failed => {
            let reason = state
                .deferred_reason
                .clone()
                .unwrap_or_else(|| "Failed".to_string());
            format!("FAIL {reason}")
        }
        ComponentStatus::Deferred => {
            let reason = state
                .deferred_reason
                .clone()
                .unwrap_or_else(|| "Deferred".to_string());
            format!("DEF {reason}")
        }
    }
}
