/// Assembly definition produced by adapters for runtime snapshots.
#[derive(Debug, Clone)]
pub struct Assembly {
    pub steps: Vec<AssemblyStepDef>,
}

/// Step definition within an assembly.
#[derive(Debug, Clone)]
pub struct AssemblyStepDef {
    pub id: String,
    pub kind: String,
    pub depends_on: Vec<String>,
    pub provides: Vec<String>,
    pub domain: String,
    pub pod: Option<String>,
    pub has_gates: bool,
}
