use primer::application::flows::reconcile::visualize::LayerType;
use primer::application::flows::reconcile::visualize::layer::determine_layer;
use primer::domain::models::assembly::Step;
use primer::domain::models::module::spec::ModuleSpec;
use phenome_ports::{ComponentState, ComponentStatus};
use std::collections::{HashMap, HashSet};
use std::time::Duration;

use crate::bootstrap::utils::layer_from_domain;

#[derive(Clone)]
pub enum TreeLine {
    Layer {
        layer: LayerType,
        total: usize,
        completed: usize,
    },
    Component {
        id: String,
        status: ComponentStatus,
        elapsed: Option<Duration>,
    },
}

pub fn build_tree_lines(
    assembly: &primer::domain::models::assembly::Assembly,
    states: &HashMap<String, ComponentState>,
    collapsed_layers: &HashSet<LayerType>,
    registry_specs: &HashMap<String, ModuleSpec>,
) -> Vec<TreeLine> {
    let mut lines = Vec::new();
    let mut seen_layers = HashSet::new();
    let mut layer_order = Vec::new();
    let mut layer_steps: HashMap<LayerType, Vec<&Step>> = HashMap::new();

    for step in &assembly.steps {
        let layer = layer_for_step(step, registry_specs);
        layer_steps.entry(layer).or_default().push(step);
        if seen_layers.insert(layer) {
            layer_order.push(layer);
        }
    }

    for layer in layer_order {
        let Some(steps) = layer_steps.get(&layer) else {
            continue;
        };
        let total = steps.len();
        let completed = steps
            .iter()
            .filter(|step| {
                states
                    .get(&step.id)
                    .map(|state| state.status == ComponentStatus::Complete)
                    .unwrap_or(false)
            })
            .count();

        lines.push(TreeLine::Layer {
            layer,
            total,
            completed,
        });

        if collapsed_layers.contains(&layer) {
            continue;
        }

        for step in steps {
            let status = states
                .get(&step.id)
                .map(|s| s.status)
                .unwrap_or(ComponentStatus::Pending);
            let elapsed = states
                .get(&step.id)
                .and_then(|s| s.timing.current_elapsed());
            lines.push(TreeLine::Component {
                id: step.id.clone(),
                status,
                elapsed,
            });
        }
    }

    lines
}

fn layer_for_step(step: &Step, registry_specs: &HashMap<String, ModuleSpec>) -> LayerType {
    if let Some(spec) = registry_specs.get(&step.id) {
        let layer = layer_from_domain(spec.domain.as_ref());
        if layer != LayerType::Unknown {
            return layer;
        }
    }
    determine_layer(step)
}
