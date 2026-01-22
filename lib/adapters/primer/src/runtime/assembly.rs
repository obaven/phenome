use std::sync::Arc;

use super::health::LiveStatus;
use crate::mapping;
use anyhow::{Context, Result};
use bootstrappo_api::contract::assembly::Assembly as BootstrappoAssembly;
use bootstrappo_api::contract::config::Config as BootstrappoConfig;
use phenome_domain::{Assembly, AssemblyStepDef};
use phenome_ports::AssemblyPort;

#[derive(Clone)]
pub struct BootstrappoAssemblyPort {
    assembly: Option<Assembly>,
    raw_assembly: Option<BootstrappoAssembly>,
    assembly_error: Option<String>,
    live_status: Option<LiveStatus>,
    config: Arc<BootstrappoConfig>,
}

impl BootstrappoAssemblyPort {
    pub fn load(live_status: Option<LiveStatus>, config: Arc<BootstrappoConfig>) -> Self {
        let (raw_assembly, assembly_error) = match build_assembly(config.as_ref()) {
            Ok(assembly) => (Some(assembly), None),
            Err(err) => (None, Some(err.to_string())),
        };
        let assembly = raw_assembly.as_ref().map(map_assembly);
        Self {
            assembly,
            raw_assembly,
            assembly_error,
            live_status,
            config,
        }
    }

    pub fn assembly(&self) -> Option<Assembly> {
        self.assembly.clone()
    }

    pub fn assembly_error(&self) -> Option<String> {
        self.assembly_error.clone()
    }

    pub fn bootstrappo_assembly(&self) -> Option<BootstrappoAssembly> {
        self.raw_assembly.clone()
    }
}

impl AssemblyPort for BootstrappoAssemblyPort {
    fn assembly(&self) -> Option<Assembly> {
        self.assembly.clone()
    }

    fn assembly_error(&self) -> Option<String> {
        self.assembly_error.clone()
    }

    fn step_readiness(&self) -> std::collections::HashMap<String, bool> {
        let mut readiness = std::collections::HashMap::new();
        let Some(raw_assembly) = &self.raw_assembly else {
            return readiness;
        };
        let cache = self.live_status.as_ref().and_then(|live| live.cache());
        for step in &raw_assembly.steps {
            let ready = if let Some(cache) = &cache {
                mapping::checks_ready(cache, step, Some(self.config.as_ref()))
            } else {
                step.checks.is_empty()
            };
            readiness.insert(step.id.clone(), ready);
        }
        readiness
    }
}

fn build_assembly(config: &BootstrappoConfig) -> Result<BootstrappoAssembly> {
    let modules = primer::application::runtime::registry::get_all_modules(config);
    primer::application::composition::assembly::builder::AssemblyBuilder::new(config.clone())
        .with_modules(modules)
        .build()
        .context("Failed to build assembly from config")
}

fn map_assembly(assembly: &BootstrappoAssembly) -> Assembly {
    let spec_map = mapping::module_specs();
    let steps = assembly
        .steps
        .iter()
        .map(|step| {
            let (domain, namespace) = spec_map
                .get(step.id.as_str())
                .cloned()
                .unwrap_or_else(|| ("unknown".to_string(), None));
            let pod = mapping::derive_pod_value(step, namespace.as_deref());
            AssemblyStepDef {
                id: step.id.clone(),
                kind: step.kind.clone(),
                depends_on: step.required.clone(),
                provides: step.provides.clone(),
                domain,
                pod,
                has_gates: !step.checks.is_empty(),
            }
        })
        .collect();
    Assembly { steps }
}
