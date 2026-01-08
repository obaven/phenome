use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use rotappo_domain::{
    ActionId, ActionRegistry, ActionStatus, Capability, CapabilityStatus, ComponentHealthStatus,
    Event, EventLevel, HealthSnapshot, HealthStatus, AssemblyStep, AssemblyStepStatus,
    AssemblySummary, Snapshot,
};
use rotappo_ui_presentation::formatting;
use rotappo_ui_terminal::{
    format_actions, format_assembly, format_events, format_problems, format_snapshot, OutputMode,
};

fn fixture_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("cli")
}

fn update_snapshots() -> bool {
    std::env::var_os("UPDATE_CLI_SNAPSHOTS").is_some()
}

fn assert_fixture(name: &str, actual: &str) {
    let path = fixture_root().join(name);
    if update_snapshots() {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .unwrap_or_else(|err| panic!("failed to create {}: {}", parent.display(), err));
        }
        fs::write(&path, actual)
            .unwrap_or_else(|err| panic!("failed to write {}: {}", path.display(), err));
        return;
    }

    let expected = fs::read_to_string(&path).unwrap_or_else(|err| {
        panic!(
            "failed to read {}: {} (run UPDATE_CLI_SNAPSHOTS=1 cargo test --test cli_golden)",
            path.display(),
            err
        )
    });

    assert!(
        expected == actual,
        "fixture mismatch for {}.\nRun UPDATE_CLI_SNAPSHOTS=1 cargo test --test cli_golden to refresh.",
        path.display()
    );
}

fn sample_snapshot() -> Snapshot {
    let mut snapshot = Snapshot {
        assembly: AssemblySummary {
            total: 0,
            completed: 0,
            in_progress: 0,
            blocked: 0,
            pending: 0,
        },
        assembly_steps: vec![
            AssemblyStep {
                id: "bootstrap".to_string(),
                kind: "apply".to_string(),
                depends_on: vec![],
                provides: vec!["cluster".to_string()],
                status: AssemblyStepStatus::Succeeded,
                domain: "core".to_string(),
                pod: Some("kube-system/boot-0".to_string()),
            },
            AssemblyStep {
                id: "secrets".to_string(),
                kind: "rotate".to_string(),
                depends_on: vec!["bootstrap".to_string()],
                provides: vec!["vault".to_string()],
                status: AssemblyStepStatus::Running,
                domain: "core".to_string(),
                pod: None,
            },
            AssemblyStep {
                id: "apps".to_string(),
                kind: "apply".to_string(),
                depends_on: vec!["secrets".to_string()],
                provides: vec!["apps".to_string()],
                status: AssemblyStepStatus::Blocked,
                domain: "edge".to_string(),
                pod: Some("apps/app-1".to_string()),
            },
        ],
        capabilities: vec![
            Capability {
                name: "Action Snapshot".to_string(),
                status: CapabilityStatus::Ready,
            },
            Capability {
                name: "Action Router".to_string(),
                status: CapabilityStatus::Degraded,
            },
            Capability {
                name: "Desktop Bridge".to_string(),
                status: CapabilityStatus::Offline,
            },
        ],
        health: HealthStatus::Degraded,
        last_updated_ms: 1_700_000_000_000,
        last_action: Some(ActionId::Reconcile),
        last_action_status: Some(ActionStatus::Running),
    };
    snapshot.update_assembly_summary_from_steps();
    snapshot
}

fn sample_health() -> HealthSnapshot {
    let mut health = HashMap::new();
    health.insert(
        "api".to_string(),
        ComponentHealthStatus::Degraded("timeout".to_string()),
    );
    HealthSnapshot {
        health,
        last_error: Some("cache init failed".to_string()),
        cache_ready: false,
    }
}

fn sample_events() -> Vec<Event> {
    vec![
        Event {
            timestamp_ms: 1_700_000_000_001,
            level: EventLevel::Info,
            message: "action loaded".to_string(),
        },
        Event {
            timestamp_ms: 1_700_000_000_500,
            level: EventLevel::Warn,
            message: "cache warming".to_string(),
        },
    ]
}

fn collect_fixtures() -> Vec<(String, String)> {
    let actions = ActionRegistry::default().actions().to_vec();
    let snapshot = sample_snapshot();
    let health = sample_health();
    let problems = formatting::problem_lines(&snapshot, Some(&health));
    let events = sample_events();

    let mut fixtures = Vec::new();
    for mode in [OutputMode::Plain, OutputMode::Json, OutputMode::Ndjson] {
        fixtures.push((
            format!("actions.{}.txt", mode.as_str()),
            format_actions(mode, &actions).expect("format actions"),
        ));
        fixtures.push((
            format!("snapshot.{}.txt", mode.as_str()),
            format_snapshot(mode, &snapshot).expect("format snapshot"),
        ));
        fixtures.push((
            format!("assembly.{}.txt", mode.as_str()),
            format_assembly(mode, &snapshot).expect("format assembly"),
        ));
        fixtures.push((
            format!("problems.{}.txt", mode.as_str()),
            format_problems(mode, &problems).expect("format problems"),
        ));
        fixtures.push((
            format!("logs.{}.txt", mode.as_str()),
            format_events(mode, &events).expect("format events"),
        ));
    }

    fixtures
}

#[test]
fn cli_output_snapshots() {
    for (name, output) in collect_fixtures() {
        assert_fixture(&name, &output);
    }
}
