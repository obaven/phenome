use rotappo_domain::{ComponentHealthStatus, HealthSnapshot, AssemblyStepStatus, Snapshot};

pub fn problem_lines(snapshot: &Snapshot, health: Option<&HealthSnapshot>) -> Vec<String> {
    let mut problems = Vec::new();
    if let Some(health) = health {
        if let Some(error) = &health.last_error {
            problems.push(format!("kube: {}", error));
        }
        problems.extend(health_problem_lines(&health.health));
        if !health.cache_ready {
            problems.push("kube cache not ready".to_string());
        }
    } else {
        problems.push("live status disabled".to_string());
    }

    for step in &snapshot.assembly_steps {
        if step.status == AssemblyStepStatus::Blocked {
            problems.push(format!("blocked: {} waiting on {:?}", step.id, step.depends_on));
        }
    }

    problems
}

fn health_problem_lines(
    health: &std::collections::HashMap<String, ComponentHealthStatus>,
) -> Vec<String> {
    let mut problems = Vec::new();
    for (name, status) in health {
        match status {
            ComponentHealthStatus::Healthy => {}
            ComponentHealthStatus::Degraded(msg) => {
                problems.push(format!("{} degraded: {}", name, msg))
            }
            ComponentHealthStatus::Unhealthy(msg) => {
                problems.push(format!("{} unhealthy: {}", name, msg))
            }
        }
    }
    problems
}
