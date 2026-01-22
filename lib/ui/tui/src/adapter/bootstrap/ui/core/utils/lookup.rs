use crate::bootstrap::state::BootstrapUiState;
use phenome_ports::PortSet;

pub fn find_dependents(
    assembly: &primer::domain::models::assembly::Assembly,
    target: &str,
) -> Vec<String> {
    assembly
        .steps
        .iter()
        .filter(|step| step.required.iter().any(|dep| dep == target))
        .map(|step| step.id.clone())
        .collect()
}

pub fn selected_component_label(ports: &PortSet, ui: &BootstrapUiState) -> Option<String> {
    ports
        .bootstrap
        .dependency_graph()
        .steps
        .get(ui.status_selected)
        .map(|step| step.id.clone())
}
