use ratatui::layout::Rect;
use ratatui::prelude::Frame;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Wrap};

use crate::app::App;

use super::helpers::{DepCategory, classify_dependency};

pub(super) fn render_registry_detail(
    frame: &mut Frame,
    area: Rect,
    app: &App,
    spec_name: &str,
) -> bool {
    let specs = app.context.ports.bootstrap.registry_specs();
    let Some(spec) = specs.get(spec_name) else {
        let paragraph = Paragraph::new(vec![Line::from(format!(
            "Unknown Registry Module: {spec_name}"
        ))])
        .wrap(Wrap { trim: true })
        .scroll((app.ui.detail_scroll, 0));
        frame.render_widget(paragraph, area);
        return true;
    };

    let mut lines = Vec::new();
    lines.push(Line::from(vec![
        Span::styled("Registry Module: ", Style::default().fg(Color::LightCyan)),
        Span::styled(spec.name, Style::default().add_modifier(Modifier::BOLD)),
    ]));

    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::raw("Description: "),
        Span::styled(spec.description, Style::default().fg(Color::White)),
    ]));
    lines.push(Line::from(vec![
        Span::raw("Domain:      "),
        Span::styled(spec.domain, Style::default().fg(Color::Cyan)),
    ]));
    lines.push(Line::from(vec![
        Span::raw("Version:     "),
        Span::raw(spec.version),
    ]));
    lines.push(Line::from(vec![
        Span::raw("Maintainer:  "),
        Span::raw(spec.maintainer),
    ]));
    if let Some(url) = spec.url.as_ref() {
        lines.push(Line::from(vec![
            Span::raw("URL:         "),
            Span::styled(*url, Style::default().fg(Color::Blue)),
        ]));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "Requirements:",
        Style::default().fg(Color::LightYellow),
    )));
    if spec.required.is_empty() {
        lines.push(Line::from("  (None)"));
    } else {
        for req in spec.required.iter() {
            let icon = match classify_dependency(req) {
                DepCategory::Security => "🔒",
                DepCategory::Database => "🔌",
                DepCategory::Storage => "💾",
                DepCategory::Infrastructure => "🏗️",
                DepCategory::Network => "🌐",
                DepCategory::Other => "📦",
            };
            lines.push(Line::from(format!("  {icon} reg:{req}")));
        }
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "Emits Capabilities (Provides):",
        Style::default().fg(Color::LightGreen),
    )));
    if spec.provides.is_empty() {
        lines.push(Line::from("  (None)"));
    } else {
        for prov in spec.provides.iter() {
            lines.push(Line::from(format!("  ✨ {prov}")));
        }
    }

    let paragraph = Paragraph::new(lines)
        .wrap(Wrap { trim: true })
        .scroll((app.ui.detail_scroll, 0));
    frame.render_widget(paragraph, area);
    true
}
