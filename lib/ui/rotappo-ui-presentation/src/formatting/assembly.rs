use std::collections::BTreeMap;

use serde::Serialize;

use phenome_domain::{AssemblyStep, Snapshot};

#[derive(Debug, Clone, Serialize)]
pub struct AssemblyStepInfo {
    pub index: usize,
    pub step: AssemblyStep,
}

#[derive(Debug, Clone, Serialize)]
pub struct AssemblyGroup {
    pub domain: String,
    pub steps: Vec<AssemblyStepInfo>,
}

pub fn assembly_groups(snapshot: &Snapshot) -> Vec<AssemblyGroup> {
    let mut grouped: BTreeMap<String, Vec<AssemblyStepInfo>> = BTreeMap::new();
    for (index, step) in snapshot.assembly_steps.iter().cloned().enumerate() {
        grouped
            .entry(step.domain.clone())
            .or_default()
            .push(AssemblyStepInfo { index, step });
    }

    grouped
        .into_iter()
        .map(|(domain, steps)| AssemblyGroup { domain, steps })
        .collect()
}
