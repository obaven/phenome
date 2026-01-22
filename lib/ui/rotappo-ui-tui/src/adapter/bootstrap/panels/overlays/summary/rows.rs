use ratatui::widgets::Row;
use phenome_ports::{AccessUrlInfo, ComponentState};
use std::collections::HashMap;
use std::time::Duration;

use crate::bootstrap::utils::format_duration;

pub(super) fn build_access_rows(urls: &[AccessUrlInfo]) -> Vec<Row<'static>> {
    let mut rows = Vec::new();
    rows.push(Row::new(vec![
        "Service".to_string(),
        "URL".to_string(),
        "Status".to_string(),
    ]));

    if urls.is_empty() {
        rows.push(Row::new(vec![
            "No access URLs discovered".to_string(),
            "-".to_string(),
            "-".to_string(),
        ]));
        return rows;
    }

    for info in urls {
        rows.push(Row::new(vec![
            info.service.clone(),
            info.url.clone(),
            info.status.label().to_string(),
        ]));
    }
    rows
}

pub(super) fn build_timing_rows(states: &HashMap<String, ComponentState>) -> Vec<Row<'static>> {
    let mut render = Duration::ZERO;
    let mut apply = Duration::ZERO;
    let mut wait = Duration::ZERO;
    for state in states.values() {
        render += state.timing.render_duration.unwrap_or_default();
        apply += state.timing.apply_duration.unwrap_or_default();
        wait += state.timing.wait_duration.unwrap_or_default();
    }

    vec![
        Row::new(vec![
            "Phase".to_string(),
            "Duration".to_string(),
            "Notes".to_string(),
        ]),
        Row::new(vec![
            "Render".to_string(),
            format_duration(render),
            if render == Duration::ZERO {
                "n/a".to_string()
            } else {
                "".to_string()
            },
        ]),
        Row::new(vec![
            "Apply".to_string(),
            format_duration(apply),
            if apply == Duration::ZERO {
                "n/a".to_string()
            } else {
                "".to_string()
            },
        ]),
        Row::new(vec![
            "Wait".to_string(),
            format_duration(wait),
            if wait == Duration::ZERO {
                "n/a".to_string()
            } else {
                "".to_string()
            },
        ]),
    ]
}

pub(super) fn build_hotspot_rows(states: &HashMap<String, ComponentState>) -> Vec<Row<'static>> {
    let mut durations: Vec<_> = states
        .values()
        .filter_map(|state| state.timing.total_duration.map(|d| (state.id.clone(), d)))
        .collect();
    durations.sort_by_key(|(_, duration)| std::cmp::Reverse(*duration));
    durations.truncate(5);

    let mut rows = Vec::new();
    rows.push(Row::new(vec![
        "Component".to_string(),
        "Total Time".to_string(),
        "Wait".to_string(),
    ]));
    for (id, duration) in durations {
        rows.push(Row::new(vec![
            id,
            format_duration(duration),
            "-".to_string(),
        ]));
    }
    rows
}
