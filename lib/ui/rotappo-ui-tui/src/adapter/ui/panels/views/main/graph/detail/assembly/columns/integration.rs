use ratatui::layout::Rect;
use ratatui::prelude::Frame;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Wrap};

use crate::app::App;
use phenome_domain::AssemblyStep;

use super::access::{gather_ingress_urls, gather_ip_info};
use super::ProvisionSets;
use super::super::super::helpers::{classify_dependency, DepCategory};

pub(super) fn render_integration(
    frame: &mut Frame,
    area: Rect,
    app: &App,
    step: &AssemblyStep,
    provisions: &ProvisionSets<'_>,
) {
    let mut lines = Vec::new();
    lines.push(Line::from(vec![Span::styled(
        "Integration & Access",
        Style::default()
            .fg(Color::LightRed)
            .add_modifier(Modifier::BOLD),
    )]));

    lines.push(Line::from(Span::styled(
        "Access & Network:",
        Style::default()
            .fg(Color::LightGreen)
            .add_modifier(Modifier::BOLD),
    )));

    let ingress_urls = gather_ingress_urls(app, step);
    let ip_info = gather_ip_info(app, step);
    let mut access_shown = false;

    if !ingress_urls.is_empty() {
        for url in &ingress_urls {
            lines.push(Line::from(format!("  🌐 {url}")));
        }
        access_shown = true;
    }

    if let Some(ip) = ip_info {
        lines.push(Line::from(format!("  📡 {ip}")));
        access_shown = true;
    }

    if !access_shown {
        lines.push(Line::from(Span::styled(
            "  (No network endpoints exposed)",
            Style::default().fg(Color::DarkGray),
        )));
    }

    lines.push(Line::from(""));

    if !provisions.admin_creds.is_empty() {
        lines.push(Line::from(Span::styled(
            "🔑 Admin Access:",
            Style::default().fg(Color::LightYellow),
        )));
        for entry in &provisions.admin_creds {
            lines.push(Line::from(format!("  ➢ {entry}")));
        }
        lines.push(Line::from(""));
    }

    lines.push(Line::from(Span::styled(
        "Dependencies:",
        Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
    )));
    if step.depends_on.is_empty() {
        lines.push(Line::from(Span::styled(
            "  (None)",
            Style::default().fg(Color::DarkGray),
        )));
    } else {
        for dep in &step.depends_on {
            let icon = match classify_dependency(dep) {
                DepCategory::Security => "🔒",
                DepCategory::Database => "🔌",
                DepCategory::Storage => "💾",
                DepCategory::Network => "🌐",
                DepCategory::Infrastructure => "🏗️",
                DepCategory::Other => "📦",
            };
            lines.push(Line::from(format!("  {icon} {dep}")));
        }
    }

    let paragraph = Paragraph::new(lines)
        .wrap(Wrap { trim: true })
        .scroll((app.ui.detail_scroll, 0));
    frame.render_widget(paragraph, area);
}
