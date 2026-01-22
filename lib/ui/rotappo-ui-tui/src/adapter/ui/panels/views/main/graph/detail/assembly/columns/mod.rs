use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::prelude::Frame;

use crate::app::App;
use phenome_domain::AssemblyStep;

mod capabilities;
mod access;
mod integration;
mod metadata;

pub(super) fn render_columns(frame: &mut Frame, area: Rect, app: &App, step: &AssemblyStep) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25),
            Constraint::Percentage(45),
            Constraint::Percentage(30),
        ])
        .split(area);

    let provisions = split_provisions(step);
    metadata::render_metadata(frame, chunks[0], app, step);
    integration::render_integration(frame, chunks[1], app, step, &provisions);
    capabilities::render_capabilities(frame, chunks[2], app, &provisions);
}

pub(super) struct ProvisionSets<'a> {
    pub(super) admin_creds: Vec<&'a str>,
    pub(super) other_provs: Vec<&'a str>,
}

fn split_provisions(step: &AssemblyStep) -> ProvisionSets<'_> {
    let mut admin_creds = Vec::new();
    let mut other_provs = Vec::new();

    for prov in &step.provides {
        let p_lower = prov.to_lowercase();
        if p_lower.contains("admin")
            || p_lower.contains("password")
            || p_lower.contains("cred")
            || p_lower.contains("login")
            || p_lower.contains("user")
            || p_lower.contains("token")
            || p_lower.contains("secret")
            || p_lower.contains("key")
        {
            admin_creds.push(prov.as_str());
        } else {
            other_provs.push(prov.as_str());
        }
    }

    ProvisionSets {
        admin_creds,
        other_provs,
    }
}
