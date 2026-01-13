use ratatui::{
    layout::{Margin, Rect},
    prelude::Frame,
    style::{Color, Modifier, Style},
    symbols::Marker,
    text::{Line, Span},
    widgets::canvas::{Canvas, Rectangle},
    widgets::{
        Block, Borders, List, ListItem, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState,
        Wrap,
    },
};

use crate::app::{App, NavView};
use crate::panels::analytics;
use crate::util::{assembly_lines, capability_icon, collect_problems, format_age};
use bootstrappo::application::flows::reconcile::visualize;
use bootstrappo::application::readiness::ResourceStatus;
use rotappo_domain::{ActionSafety, AssemblyStepStatus, EventLevel};
use rotappo_ui_presentation::formatting;

pub fn render_main(frame: &mut Frame, area: Rect, app: &mut App) {
    app.ui.body_area = area;
    reset_panel_areas(app);
    app.graph.clear_request();

    let mut title = app.active_nav().title().to_string();
    if matches!(
        app.active_view(),
        NavView::TopologyDagGraph | NavView::TopologyDualGraph
    ) {
        let term = std::env::var("TERM").unwrap_or("?".to_string());
        let hover = app.ui.hover_node_id.as_deref().unwrap_or("-");
        let node_count = app.graph.layout().map(|l| l.nodes.len()).unwrap_or(0);
        title = format!(
            "{} [Proto:{} Img:{} TERM:{} Hover:{} Nodes:{} Details:{}]",
            title,
            app.graph.protocol_label(),
            app.graph.image_active(),
            term,
            hover,
            node_count,
            app.ui.show_detail_panel
        );
    }

    let block = Block::default()
        .title(Span::styled(
            title,
            Style::default().add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    // Only set opaque background for non-graph views to allow z=-1 image to show through
    let block = if matches!(
        app.active_view(),
        NavView::TopologyDagGraph | NavView::TopologyDualGraph
    ) {
        block.style(Style::default().bg(Color::Reset))
    } else {
        block.style(Style::default().bg(Color::Rgb(18, 20, 24)))
    };

    let inner = block.inner(area);
    frame.render_widget(block, area);

    match app.active_view() {
        NavView::AnalyticsRealtime => analytics::render_realtime(frame, inner, app),
        NavView::AnalyticsHistorical => analytics::render_historical(frame, inner, app),
        NavView::AnalyticsPredictions => analytics::render_predictions(frame, inner, app),
        NavView::AnalyticsRecommendations => analytics::render_recommendations(frame, inner, app),
        NavView::AnalyticsInsights => analytics::render_insights(frame, inner, app),
        NavView::TopologyAssembly => render_topology_assembly(frame, inner, app),
        NavView::TopologyDomains => render_topology_domains(frame, inner, app),
        NavView::TopologyCapabilities => render_topology_capabilities(frame, inner, app),
        NavView::TopologyQueue => render_topology_queue(frame, inner, app),
        NavView::TopologyHealth => render_topology_health(frame, inner, app),
        NavView::TopologyDagGraph => {
            let label = format!(
                "DAG Graph [{}] img:{}",
                app.graph.protocol_label(),
                app.graph.image_active()
            );
            render_topology_graph(frame, inner, app, visualize::ViewType::Full, &label);
        }
        NavView::TopologyDualGraph => {
            let label = format!(
                "Dual Graph [{}] img:{}",
                app.graph.protocol_label(),
                app.graph.image_active()
            );
            render_topology_graph(frame, inner, app, visualize::ViewType::Dual, &label);
        }
        NavView::TerminalLogs => render_terminal_logs(frame, inner, app),
        NavView::TerminalEvents => render_terminal_events(frame, inner, app),
        NavView::TerminalCommands => render_terminal_commands(frame, inner, app),
        NavView::TerminalDiagnostics => render_terminal_diagnostics(frame, inner, app),
    }
}

fn reset_panel_areas(app: &mut App) {
    app.ui.actions_area = Rect::default();
    app.ui.settings_area = Rect::default();
    app.ui.settings_controls_row = None;
    app.ui.log_controls_area = Rect::default();
    app.ui.assembly_area = Rect::default();
    app.ui.assembly_progress_area = Rect::default();
    app.ui.snapshot_area = Rect::default();
    app.ui.capabilities_area = Rect::default();
    app.ui.logs_area = Rect::default();
    app.ui.problems_area = Rect::default();
    app.ui.help_area = Rect::default();
    app.ui.log_menu_area = Rect::default();
    app.ui.log_filter_tag_area = Rect::default();
    app.ui.log_stream_tag_area = Rect::default();
    app.ui.collapsed_actions = true;
    app.ui.collapsed_logs = true;
    app.ui.collapsed_problems = true;
    app.ui.collapsed_assembly_steps = true;
    app.ui.collapsed_capabilities = true;
}

#[allow(dead_code)]
fn render_analytics_overview(frame: &mut Frame, area: Rect, app: &mut App) {
    let snapshot = app.runtime.snapshot();
    let health_style = match snapshot.health.as_str() {
        "healthy" => Style::default().fg(Color::Green),
        "degraded" => Style::default().fg(Color::Yellow),
        _ => Style::default().fg(Color::Red),
    };
    let action_label = snapshot
        .last_action
        .map(|action| action.to_string())
        .unwrap_or_else(|| "none".to_string());
    let action_status = snapshot
        .last_action_status
        .map(|status| status.as_str())
        .unwrap_or("idle");
    let age = format_age(snapshot.last_updated_ms);

    let mut lines = Vec::new();
    lines.push(section_title("Overview"));
    lines.push(Line::from(vec![
        Span::raw("Health: "),
        Span::styled(snapshot.health.as_str(), health_style),
    ]));
    lines.push(Line::from(format!(
        "Assembly: {completed}/{total} ({percent}%)",
        completed = snapshot.assembly.completed,
        total = snapshot.assembly.total,
        percent = snapshot.assembly.percent_complete(),
    )));
    lines.push(Line::from(format!(
        "Running: {running}  Blocked: {blocked}  Pending: {pending}",
        running = snapshot.assembly.in_progress,
        blocked = snapshot.assembly.blocked,
        pending = snapshot.assembly.pending,
    )));
    lines.push(Line::from(format!("Last update: {age}")));
    lines.push(Line::from(format!(
        "Last action: {action_label} ({action_status})"
    )));
    lines.push(Line::from(""));
    lines.push(section_title("Signals"));
    lines.push(Line::from(format!(
        "Events cached: {count}",
        count = app.ui.log_cache.len()
    )));
    lines.push(Line::from(format!(
        "Available actions: {count}",
        count = app.runtime.registry().actions().len()
    )));
    let problems = collect_problems(app);
    lines.push(Line::from(format!(
        "Open problems: {count}",
        count = problems.len()
    )));
    lines.push(Line::from(""));
    lines.push(section_title("Top Actions"));
    if app.runtime.registry().actions().is_empty() {
        lines.push(Line::from("No actions registered."));
    } else {
        for action in app.runtime.registry().actions().iter().take(4) {
            lines.push(Line::from(format!("- {}: {}", action.id, action.label)));
        }
    }
    if !problems.is_empty() {
        lines.push(Line::from(""));
        lines.push(section_title("Problems"));
        for problem in problems.iter().take(3) {
            lines.push(Line::from(format!("- {problem}")));
        }
    }

    let paragraph = Paragraph::new(lines).wrap(Wrap { trim: true });
    frame.render_widget(paragraph, area);
}

#[allow(dead_code)]
fn render_analytics_metrics(frame: &mut Frame, area: Rect, app: &mut App) {
    let snapshot = app.runtime.snapshot();
    let mut ready = 0;
    let mut degraded = 0;
    let mut offline = 0;
    let mut degraded_names = Vec::new();
    for capability in &snapshot.capabilities {
        match capability.status {
            rotappo_domain::CapabilityStatus::Ready => ready += 1,
            rotappo_domain::CapabilityStatus::Degraded => {
                degraded += 1;
                if degraded_names.len() < 5 {
                    degraded_names.push(capability.name.as_str());
                }
            }
            rotappo_domain::CapabilityStatus::Offline => {
                offline += 1;
                if degraded_names.len() < 5 {
                    degraded_names.push(capability.name.as_str());
                }
            }
        }
    }
    let mut lines = Vec::new();
    lines.push(section_title("Capabilities"));
    lines.push(Line::from(format!(
        "Ready: {ready}  Degraded: {degraded}  Offline: {offline}"
    )));
    if !degraded_names.is_empty() {
        lines.push(Line::from(""));
        lines.push(section_title("Attention"));
        for name in degraded_names {
            lines.push(Line::from(format!("- {name}")));
        }
    }
    let paragraph = Paragraph::new(lines).wrap(Wrap { trim: true });
    frame.render_widget(paragraph, area);
}

#[allow(dead_code)]
fn render_analytics_actions(frame: &mut Frame, area: Rect, app: &mut App) {
    render_action_list(frame, area, app);
}

#[allow(dead_code)]
fn render_analytics_problems(frame: &mut Frame, area: Rect, app: &mut App) {
    app.ui.problems_area = area;
    app.ui.collapsed_problems = false;
    let problems = collect_problems(app);
    let mut lines = Vec::new();
    lines.push(section_title("Problem Feed"));
    if problems.is_empty() {
        lines.push(Line::from("No problems detected."));
    } else {
        for problem in problems {
            lines.push(Line::from(format!("- {problem}")));
        }
    }
    let paragraph = Paragraph::new(lines).wrap(Wrap { trim: true });
    frame.render_widget(paragraph, area);
}

#[allow(dead_code)]
fn render_analytics_events(frame: &mut Frame, area: Rect, app: &mut App) {
    let mut info = 0;
    let mut warn = 0;
    let mut error = 0;
    for event in &app.ui.log_cache {
        match event.level {
            EventLevel::Info => info += 1,
            EventLevel::Warn => warn += 1,
            EventLevel::Error => error += 1,
        }
    }
    let mut lines = Vec::new();
    lines.push(section_title("Event Pulse"));
    lines.push(Line::from(format!(
        "Info: {info}  Warn: {warn}  Error: {error}"
    )));
    lines.push(Line::from(""));
    lines.push(section_title("Recent"));
    for event in app.ui.log_cache.iter().rev().take(8) {
        let age = format_age(event.timestamp_ms);
        let level_style = match event.level {
            EventLevel::Info => Style::default().fg(Color::Cyan),
            EventLevel::Warn => Style::default().fg(Color::Yellow),
            EventLevel::Error => Style::default().fg(Color::Red),
        };
        lines.push(Line::from(vec![
            Span::styled(event.level.as_str(), level_style),
            Span::raw(" "),
            Span::raw(event.message.as_str()),
            Span::raw(" "),
            Span::styled(format!("({age})"), Style::default().fg(Color::DarkGray)),
        ]));
    }
    let paragraph = Paragraph::new(lines).wrap(Wrap { trim: true });
    frame.render_widget(paragraph, area);
}

#[allow(dead_code)]
fn render_analytics_config(frame: &mut Frame, area: Rect, app: &mut App) {
    let mut lines = Vec::new();
    lines.push(section_title("Runtime"));
    lines.push(Line::from(format!("Host: {}", app.context.host_domain)));
    lines.push(Line::from(format!(
        "Config: {}",
        app.context.config_path.display()
    )));
    lines.push(Line::from(format!(
        "Assembly: {}",
        app.context.assembly_path.display()
    )));
    if let Some(error) = &app.context.assembly_error {
        lines.push(Line::from(""));
        lines.push(section_title("Assembly Error"));
        lines.push(Line::from(error.to_string()));
    }
    if let Some(error) = &app.context.live_status_error {
        lines.push(Line::from(""));
        lines.push(section_title("Live Status Error"));
        lines.push(Line::from(error.to_string()));
    }
    let paragraph = Paragraph::new(lines).wrap(Wrap { trim: true });
    frame.render_widget(paragraph, area);
}

fn render_topology_assembly(frame: &mut Frame, area: Rect, app: &mut App) {
    app.ui.assembly_area = area;
    app.ui.collapsed_assembly_steps = false;
    let lines = assembly_lines(app.runtime.snapshot())
        .into_iter()
        .map(|entry| entry.line)
        .collect::<Vec<_>>();
    let paragraph = if lines.is_empty() {
        Paragraph::new("No assembly data available.").wrap(Wrap { trim: true })
    } else {
        Paragraph::new(lines).wrap(Wrap { trim: true })
    };
    let paragraph = paragraph.scroll((app.ui.assembly_scroll, 0));
    frame.render_widget(paragraph, area);
}

fn render_topology_domains(frame: &mut Frame, area: Rect, app: &mut App) {
    let snapshot = app.runtime.snapshot();
    let mut lines = Vec::new();
    lines.push(section_title("Domains"));
    let groups = formatting::assembly_groups(snapshot);
    if groups.is_empty() {
        lines.push(Line::from("No domain data available."));
    } else {
        for group in groups {
            lines.push(Line::from(format!(
                "- {} ({})",
                group.domain.as_str(),
                group.steps.len()
            )));
        }
    }
    let paragraph = Paragraph::new(lines).wrap(Wrap { trim: true });
    frame.render_widget(paragraph, area);
}

fn render_topology_capabilities(frame: &mut Frame, area: Rect, app: &mut App) {
    app.ui.capabilities_area = area;
    app.ui.collapsed_capabilities = false;
    let snapshot = app.runtime.snapshot();
    let mut lines = Vec::new();
    lines.push(section_title("Capabilities"));
    if snapshot.capabilities.is_empty() {
        lines.push(Line::from("No capabilities available."));
    } else {
        for capability in &snapshot.capabilities {
            let icon = capability_icon(capability.status);
            lines.push(Line::from(format!(
                "[{icon}] {} ({})",
                capability.name,
                capability.status.as_str()
            )));
        }
    }
    let paragraph = Paragraph::new(lines).wrap(Wrap { trim: true });
    frame.render_widget(paragraph, area);
}

fn render_topology_queue(frame: &mut Frame, area: Rect, app: &mut App) {
    let snapshot = app.runtime.snapshot();
    let mut lines = Vec::new();
    lines.push(section_title("Queue State"));
    lines.push(Line::from(format!(
        "Ready: {ready}  Running: {running}",
        ready = snapshot.assembly.completed,
        running = snapshot.assembly.in_progress
    )));
    lines.push(Line::from(format!(
        "Blocked: {blocked}  Pending: {pending}",
        blocked = snapshot.assembly.blocked,
        pending = snapshot.assembly.pending
    )));
    lines.push(Line::from(""));
    lines.push(section_title("Blocked Steps"));
    let mut blocked = Vec::new();
    for group in formatting::assembly_groups(snapshot) {
        for step in group.steps {
            if step.step.status == AssemblyStepStatus::Blocked {
                blocked.push(step.step.id);
            }
        }
    }
    if blocked.is_empty() {
        lines.push(Line::from("No blocked steps."));
    } else {
        for id in blocked.into_iter().take(6) {
            lines.push(Line::from(format!("- {id}")));
        }
    }
    let paragraph = Paragraph::new(lines).wrap(Wrap { trim: true });
    frame.render_widget(paragraph, area);
}

fn render_topology_health(frame: &mut Frame, area: Rect, app: &mut App) {
    let snapshot = app.runtime.snapshot();
    let mut lines = Vec::new();
    lines.push(section_title("Health"));
    lines.push(Line::from(format!("Status: {}", snapshot.health.as_str())));
    lines.push(Line::from(""));
    lines.push(section_title("Problems"));
    let problems = collect_problems(app);
    if problems.is_empty() {
        lines.push(Line::from("No problems detected."));
    } else {
        for problem in problems {
            lines.push(Line::from(format!("- {problem}")));
        }
    }
    let paragraph = Paragraph::new(lines).wrap(Wrap { trim: true });
    frame.render_widget(paragraph, area);
}

fn render_topology_graph(
    frame: &mut Frame,
    area: Rect,
    app: &mut App,
    view: visualize::ViewType,
    label: &str,
) {
    // Layout: if detail panel open, split into Main (Graph) and Bottom (Details)
    let (graph_area, sidebar_area) = if app.ui.show_detail_panel {
        let chunks = ratatui::layout::Layout::default()
            .direction(ratatui::layout::Direction::Vertical)
            .constraints([
                ratatui::layout::Constraint::Min(10),
                ratatui::layout::Constraint::Length(15), // Increased height slightly
            ])
            .split(area);
        app.ui.detail_area = chunks[1];
        (chunks[0], Some(chunks[1]))
    } else {
        app.ui.detail_area = Rect::default();
        (area, None)
    };

    app.ui.assembly_area = graph_area;
    app.ui.collapsed_assembly_steps = false;
    let assembly = app.context.ports.bootstrap.dependency_graph();
    let (graph, node_map) = visualize::graph::build_filtered_graph(assembly, view);

    // Invert the map for DOT generation
    let index_map: std::collections::HashMap<_, _> =
        node_map.iter().map(|(k, v)| (*v, k.clone())).collect();

    let dot = visualize::render::generate_pretty_dot(&graph, &index_map);

    if let Err(error) = app.graph.ensure_layout(&dot) {
        app.graph.mark_layout_failed(error.to_string());
    }

    // Explicitly queue image rendering request
    app.graph.queue_request(graph_area, dot.clone());

    if let Some(layout) = app.graph.layout() {
        let bounds = app.graph.view_bounds_for(layout, graph_area);
        let selected = app.graph.selected_id();
        let dependency = selected
            .map(|id| layout.dependency_paths(id))
            .unwrap_or_default();
        let selected_id_clone = selected.map(|s| s.to_string());
        let image_active = app.graph.image_active();

        let canvas = Canvas::default()
            .marker(Marker::Braille)
            .x_bounds([bounds.x_min, bounds.x_max])
            .y_bounds([bounds.y_min, bounds.y_max])
            .paint(move |ctx| {
                // If image is NOT active, render the graph using Braille
                if !image_active {
                    // Draw edges
                    for (i, edge) in layout.edges.iter().enumerate() {
                        let color = if dependency.edges.contains(&i) {
                            Color::Cyan
                        } else {
                            Color::Gray
                        };
                        for i in 0..edge.points.len().saturating_sub(1) {
                            let p1 = edge.points[i];
                            let p2 = edge.points[i + 1];
                            ctx.draw(&ratatui::widgets::canvas::Line {
                                x1: p1.0,
                                y1: p1.1,
                                x2: p2.0,
                                y2: p2.1,
                                color,
                            });
                        }
                    }

                    // Draw nodes
                    for (i, node) in layout.nodes.iter().enumerate() {
                        let color = if selected_id_clone.as_deref() == Some(node.id.as_str()) {
                            Color::Yellow
                        } else if dependency.nodes.contains(&i) {
                            Color::Cyan
                        } else {
                            Color::Blue
                        };

                        let rect = Rectangle {
                            x: node.x - node.width / 2.0,
                            y: node.y - node.height / 2.0,
                            width: node.width,
                            height: node.height,
                            color,
                        };
                        ctx.draw(&rect);
                    }
                } else {
                    // Draw selection highlight over image
                    if let Some(selected_id) = selected {
                        if let Some(node) = layout.node(selected_id) {
                            let rect = Rectangle {
                                x: node.x - node.width / 2.0,
                                y: node.y - node.height / 2.0,
                                width: node.width,
                                height: node.height,
                                color: Color::Yellow,
                            };
                            ctx.draw(&rect);
                        }
                    }
                }
            });
        frame.render_widget(canvas, graph_area);

        // Render Detail Sidebar
        if let Some(sidebar) = sidebar_area {
            render_detail_sidebar(frame, sidebar, app);
        }

        // Render Search Overlay
        if app.ui.search_active {
            let search_area = Rect {
                x: graph_area.x + 2,
                y: graph_area.y + 1,
                width: 40,
                height: 3,
            };
            let block = Block::default()
                .title("Search Node")
                .borders(Borders::ALL)
                .style(Style::default().bg(Color::Blue).fg(Color::White));
            frame.render_widget(ratatui::widgets::Clear, search_area);
            let paragraph = Paragraph::new(app.ui.search_query.as_str()).block(block);
            frame.render_widget(paragraph, search_area);
            // set cursor? Frame doesn't support set_cursor easily without wrapping Logic?
            // Actually terminal backend does. But we are in render loop.
            // app can hint cursor pos?
            // We'll skip cursor for now, text is enough.
        }

        return;
    }

    let mut lines = Vec::new();
    lines.push(section_title(label));
    if let Some(error) = app.graph.layout_error() {
        lines.push(Line::from(format!("Interactive layout failed: {error}")));
        lines.push(Line::from(""));
    }
    for line in dot.lines() {
        lines.push(Line::from(line.to_string()));
    }
    let paragraph = Paragraph::new(lines)
        .wrap(Wrap { trim: false })
        .scroll((app.ui.assembly_scroll, 0));
    frame.render_widget(paragraph, area);
}

fn classify_dependency(dep: &str) -> DepCategory {
    let d = dep.to_lowercase();
    if d.contains("postgres")
        || d.contains("redis")
        || d.contains("mongo")
        || d.contains("qdrant")
        || d.contains("sql")
        || d.contains("db")
        || d.contains("data")
    {
        return DepCategory::Database;
    }
    if d.contains("minio") || d.contains("longhorn") || d.contains("s3") || d.contains("storage") {
        return DepCategory::Storage;
    }
    if d.contains("oidc")
        || d.contains("authelia")
        || d.contains("secret")
        || d.contains("cert")
        || d.contains("vault")
        || d.contains("auth")
    {
        return DepCategory::Security;
    }
    if d.contains("ingress") || d.contains("dns") || d.contains("network") || d.contains("proxy") {
        return DepCategory::Network;
    }
    if d.contains("kro") || d.contains("cnpg") || d.contains("operator") {
        return DepCategory::Infrastructure;
    }
    DepCategory::Other
}

enum DepCategory {
    Database,
    Storage,
    Security,
    Network,
    Infrastructure,
    Other,
}

fn render_detail_sidebar(frame: &mut Frame, area: Rect, app: &mut App) {
    let block = Block::default()
        .title("Details")
        .borders(Borders::TOP)
        .style(Style::default().bg(Color::Rgb(18, 20, 24)).fg(Color::White));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let mut lines = Vec::new();
    if let Some(node) = app.graph.selected_node() {
        // HACK: Handle Registry Nodes (Dual Graph View)
        if node.id.starts_with("reg:") {
            let spec_name = node.id.trim_start_matches("reg:");
            let specs = app.context.ports.bootstrap.registry_specs();

            if let Some(spec) = specs.get(spec_name) {
                // Header
                lines.push(Line::from(vec![
                    Span::styled(
                        "📦 Registry Module: ",
                        Style::default().fg(Color::LightCyan),
                    ),
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
                if let Some(url) = spec.url {
                    lines.push(Line::from(vec![
                        Span::raw("URL:         "),
                        Span::styled(url, Style::default().fg(Color::Blue)),
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
                    for req in spec.required {
                        let icon = match classify_dependency(req) {
                            DepCategory::Security => "🔒",
                            DepCategory::Database => "🔌",
                            DepCategory::Storage => "💾",
                            DepCategory::Infrastructure => "🏗️",
                            _ => "📦",
                        };
                        lines.push(Line::from(format!("  {} reg:{}", icon, req)));
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
                    for prov in spec.provides {
                        lines.push(Line::from(format!("  ✨ {}", prov)));
                    }
                }
                let p = Paragraph::new(lines)
                    .wrap(Wrap { trim: true })
                    .scroll((app.ui.detail_scroll, 0));
                frame.render_widget(p, inner);
                return;
            } else {
                lines.push(Line::from(format!(
                    "Unknown Registry Module: {}",
                    spec_name
                )));
                let p = Paragraph::new(lines)
                    .wrap(Wrap { trim: true })
                    .scroll((app.ui.detail_scroll, 0));
                frame.render_widget(p, inner);
                return;
            }
        }

        // Standard Assembly Node Logic
        let snapshot = app.runtime.snapshot();
        // Fallback to finding step in assembly using original node.id
        let step_opt = snapshot.assembly_steps.iter().find(|s| s.id == node.id);

        if let Some(step) = step_opt {
            // Layout: Top Lineage Bar + 3 Columns below
            let main_chunks = ratatui::layout::Layout::default()
                .direction(ratatui::layout::Direction::Vertical)
                .constraints([
                    ratatui::layout::Constraint::Length(2), // Lineage bar
                    ratatui::layout::Constraint::Min(1),    // Columns
                ])
                .split(inner);

            // --- Horizontal Flow Lineage ---
            let mut lineage_spans = Vec::new();
            lineage_spans.push(Span::styled(
                "Flow: ",
                Style::default()
                    .fg(Color::Magenta)
                    .add_modifier(Modifier::BOLD),
            ));

            if !step.depends_on.is_empty() {
                lineage_spans.push(Span::styled(
                    format!("[{}]", step.depends_on.join(", ")),
                    Style::default().fg(Color::Gray),
                ));
                lineage_spans.push(Span::raw(" -> "));
            } else {
                lineage_spans.push(Span::raw("(Root) -> "));
            }

            lineage_spans.push(Span::styled(
                format!("({})", step.id),
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ));
            lineage_spans.push(Span::raw(" -> "));

            let children: Vec<String> = snapshot
                .assembly_steps
                .iter()
                .filter(|s| s.depends_on.contains(&step.id))
                .map(|s| s.id.clone())
                .collect();

            if !children.is_empty() {
                lineage_spans.push(Span::styled(
                    format!("[{}]", children.join(", ")),
                    Style::default().fg(Color::Cyan),
                ));
            } else {
                lineage_spans.push(Span::raw("(Leaf)"));
            }
            frame.render_widget(Paragraph::new(Line::from(lineage_spans)), main_chunks[0]);

            // --- Columns ---
            let col_chunks = ratatui::layout::Layout::default()
                .direction(ratatui::layout::Direction::Horizontal)
                .constraints([
                    ratatui::layout::Constraint::Percentage(25), // Metadata
                    ratatui::layout::Constraint::Percentage(45), // Services & Access
                    ratatui::layout::Constraint::Percentage(30), // Capabilities
                ])
                .split(main_chunks[1]);

            // --- Col 1: Metadata ---
            let mut col1 = Vec::new();
            col1.push(Line::from(vec![Span::styled(
                "Metadata",
                Style::default()
                    .fg(Color::Blue)
                    .add_modifier(Modifier::BOLD),
            )]));
            col1.push(Line::from(format!(" ID:     {}", step.id)));
            col1.push(Line::from(format!(" Status: {}", step.status.as_str())));
            col1.push(Line::from(format!(" Domain: {}", step.domain)));
            col1.push(Line::from(format!(" Kind:   {}", step.kind)));
            if let Some(pod) = &step.pod {
                col1.push(Line::from(format!(" Pod:    {}", pod)));
            }
            let p1 = Paragraph::new(col1)
                .wrap(Wrap { trim: true })
                .scroll((app.ui.detail_scroll, 0));
            frame.render_widget(p1, col_chunks[0]);

            // --- Col 2: Service Integration & Access ---
            let mut col2 = Vec::new();
            col2.push(Line::from(vec![Span::styled(
                "Integration & Access",
                Style::default()
                    .fg(Color::LightRed)
                    .add_modifier(Modifier::BOLD),
            )]));

            // 1. Runtime Access URLs (Ingress)
            let mut ingress_urls = Vec::new();
            // Need to check generic bounds or assume AccessUrlInfo available
            let all_urls = app.context.ports.bootstrap.access_urls();

            // Fuzzy match URLs to this component
            for info in &all_urls {
                let svc_lower = info.service.to_lowercase();
                let id_lower = step.id.to_lowercase();
                // Normalized check (ignore hyphens)
                let svc_norm = svc_lower.replace('-', "");
                let id_norm = id_lower.replace('-', "");

                if svc_lower.contains(&id_lower)
                    || id_lower.contains(&svc_lower)
                    || svc_norm.contains(&id_norm)
                    || id_norm.contains(&svc_norm)
                {
                    ingress_urls.push(info.url.clone());
                }
            }

            // 2. Runtime IP (ClusterIP / LoadBalancer)
            // 2. Runtime IP (ClusterIP / LoadBalancer)
            let mut ip_info = None;
            if let Ok(details) = app.context.ports.bootstrap.get_detailed_status(&step.id) {
                if let bootstrappo::application::readiness::ResourceStatus::Service {
                    cluster_ip,
                    load_balancer_ip,
                } = details.resource_status
                {
                    if let Some(lb) = load_balancer_ip {
                        ip_info = Some(format!("LB IP: {}", lb));
                    } else if let Some(cip) = cluster_ip {
                        ip_info = Some(format!("ClusterIP: {}", cip));
                    }
                }
            }

            // 3. Static "Provides" analysis
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
                    admin_creds.push(prov);
                } else {
                    // Include everything else in capabilities!
                    other_provs.push(prov);
                }
            }

            // Render Highlights
            col2.push(Line::from(Span::styled(
                "Access & Network:",
                Style::default()
                    .fg(Color::LightGreen)
                    .add_modifier(Modifier::BOLD),
            )));

            let mut access_shown = false;

            if !ingress_urls.is_empty() {
                for u in &ingress_urls {
                    col2.push(Line::from(format!("  🌐 {}", u)));
                }
                access_shown = true;
            }

            if let Some(ip) = ip_info {
                col2.push(Line::from(format!("  📡 {}", ip)));
                access_shown = true;
            } else if !access_shown {
                // Try to look for Service explicitly if we missed it?
                // No, just show placeholder
            }

            if !access_shown {
                col2.push(Line::from(Span::styled(
                    "  (No network endpoints exposed)",
                    Style::default().fg(Color::DarkGray),
                )));
            }

            col2.push(Line::from(""));

            if !admin_creds.is_empty() {
                col2.push(Line::from(Span::styled(
                    "🔑 Admin Access:",
                    Style::default().fg(Color::LightYellow),
                )));
                for c in &admin_creds {
                    col2.push(Line::from(format!("  ➢ {}", c)));
                }
                col2.push(Line::from(""));
            }

            // Dependencies
            col2.push(Line::from(Span::styled(
                "Dependencies:",
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            )));
            if step.depends_on.is_empty() {
                col2.push(Line::from(Span::styled(
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
                    col2.push(Line::from(format!("  {} {}", icon, dep)));
                }
            }

            let p2 = Paragraph::new(col2)
                .wrap(Wrap { trim: true })
                .scroll((app.ui.detail_scroll, 0));
            frame.render_widget(p2, col_chunks[1]);

            // --- Col 3: Capabilities ---
            let mut col3 = Vec::new();
            col3.push(Line::from(vec![Span::styled(
                "Capabilities",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            )]));
            if !other_provs.is_empty() {
                for prov in other_provs {
                    col3.push(Line::from(format!("✨ {}", prov)));
                }
            } else {
                col3.push(Line::from(Span::styled(
                    "(None)",
                    Style::default().fg(Color::DarkGray),
                )));
            }
            let p3 = Paragraph::new(col3)
                .wrap(Wrap { trim: true })
                .scroll((app.ui.detail_scroll, 0));
            frame.render_widget(p3, col_chunks[2]);
        } else {
            // Fallback if step not found (e.g. ghost node)
            lines.push(Line::from(vec![
                Span::raw("Node: "),
                Span::styled(
                    &node.id,
                    Style::default()
                        .add_modifier(Modifier::BOLD)
                        .fg(Color::Yellow),
                ),
            ]));
            lines.push(Line::from(" (No assembly step details found)"));
        }
    } else {
        lines.push(Line::from("No node selected."));
        lines.push(Line::from(""));
        lines.push(Line::from("Navigation:"));
        lines.push(Line::from(" [Arrows]: Pan Graph"));
        lines.push(Line::from(" [Click]: Select Node"));
        lines.push(Line::from(" [Enter]: Toggle Panel"));
        lines.push(Line::from(" [Shift+Up/Down]: Scroll This Panel"));
    }
    let paragraph = Paragraph::new(lines)
        .wrap(Wrap { trim: true })
        .scroll((app.ui.detail_scroll, 0));
    frame.render_widget(paragraph, inner);
}

// Old function signature for partial replacement match

// truncate_label removed as unused

fn render_terminal_logs(frame: &mut Frame, area: Rect, app: &mut App) {
    app.ui.logs_area = area;
    app.ui.collapsed_logs = false;
    let mut lines = Vec::new();
    lines.push(section_title("Stream"));
    lines.push(Line::from(format!(
        "Filter: {}  Interval: {}s  Watch: {}",
        app.ui.log_config.filter.as_str(),
        app.ui.log_config.interval.as_secs(),
        if app.ui.auto_refresh { "on" } else { "off" }
    )));
    lines.push(Line::from(""));

    let events = app.filtered_events();
    if events.is_empty() {
        lines.push(Line::from("No events captured yet."));
    } else {
        for event in events {
            let level_style = match event.level {
                EventLevel::Info => Style::default().fg(Color::Cyan),
                EventLevel::Warn => Style::default().fg(Color::Yellow),
                EventLevel::Error => Style::default().fg(Color::Red),
            };
            lines.push(Line::from(vec![
                Span::styled(event.level.as_str(), level_style),
                Span::raw(" "),
                Span::raw(event.message.as_str()),
            ]));
        }
    }

    let paragraph = Paragraph::new(lines)
        .wrap(Wrap { trim: true })
        .scroll((app.ui.log_scroll, 0));
    frame.render_widget(paragraph, area);
}

fn render_terminal_events(frame: &mut Frame, area: Rect, app: &mut App) {
    app.ui.logs_area = area;
    app.ui.collapsed_logs = false;
    let mut lines = Vec::new();
    lines.push(section_title("Event Feed"));
    if app.ui.log_cache.is_empty() {
        lines.push(Line::from("No events captured yet."));
    } else {
        for event in app.ui.log_cache.iter().rev().take(12) {
            let age = format_age(event.timestamp_ms);
            let level_style = match event.level {
                EventLevel::Info => Style::default().fg(Color::Cyan),
                EventLevel::Warn => Style::default().fg(Color::Yellow),
                EventLevel::Error => Style::default().fg(Color::Red),
            };
            lines.push(Line::from(vec![
                Span::styled(event.level.as_str(), level_style),
                Span::raw(" "),
                Span::raw(event.message.as_str()),
                Span::raw(" "),
                Span::styled(format!("({age})"), Style::default().fg(Color::DarkGray)),
            ]));
        }
    }
    let paragraph = Paragraph::new(lines)
        .wrap(Wrap { trim: true })
        .scroll((app.ui.log_scroll, 0));
    frame.render_widget(paragraph, area);
}

fn render_terminal_commands(frame: &mut Frame, area: Rect, app: &mut App) {
    render_action_list(frame, area, app);
}

fn render_action_list(frame: &mut Frame, area: Rect, app: &mut App) {
    app.ui.actions_area = area;
    app.ui.collapsed_actions = false;
    let actions = app.runtime.registry().actions();
    let total_actions = actions.len();
    let view_height = area.height.max(1) as usize;
    let visible_items = view_height.max(1);
    let max_offset = total_actions.saturating_sub(visible_items);
    if app.ui.actions_scroll as usize > max_offset {
        app.ui.actions_scroll = max_offset as u16;
    }
    let offset = app.ui.actions_scroll as usize;
    let mut items = Vec::new();
    for action in actions.iter().skip(offset).take(visible_items) {
        let safety_style = match action.safety {
            ActionSafety::Safe => Style::default().fg(Color::Green),
            ActionSafety::Guarded => Style::default().fg(Color::Yellow),
            ActionSafety::Destructive => Style::default().fg(Color::Red),
        };
        let line = Line::from(vec![
            Span::styled(action.id.to_string(), Style::default().fg(Color::Cyan)),
            Span::raw(" "),
            Span::raw(action.label),
            Span::raw(" "),
            Span::styled(format!("[{}]", action.safety.as_str()), safety_style),
        ]);
        items.push(ListItem::new(line));
    }
    let mut list_state = ratatui::widgets::ListState::default();
    if let Some(selected) = app.action_state.selected() {
        if selected >= offset && selected < offset + visible_items {
            list_state.select(Some(selected - offset));
        }
    } else if !items.is_empty() {
        app.action_state.select(Some(0));
        list_state.select(Some(0));
    }
    let list = List::new(items).highlight_symbol("> ").highlight_style(
        Style::default()
            .fg(Color::Black)
            .bg(Color::White)
            .add_modifier(Modifier::BOLD),
    );
    frame.render_stateful_widget(list, area, &mut list_state);

    if total_actions > visible_items && visible_items > 0 {
        let mut state = ScrollbarState::new(total_actions).position(app.ui.actions_scroll as usize);
        let bar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .style(Style::default().fg(Color::Cyan));
        frame.render_stateful_widget(
            bar,
            area.inner(Margin {
                horizontal: 0,
                vertical: 0,
            }),
            &mut state,
        );
    }
}

fn render_terminal_diagnostics(frame: &mut Frame, area: Rect, app: &mut App) {
    let mut lines = Vec::new();
    lines.push(section_title("Diagnostics"));
    let problems = collect_problems(app);
    if problems.is_empty() {
        lines.push(Line::from("No problems detected."));
    } else {
        for problem in problems.iter().take(8) {
            lines.push(Line::from(format!("- {problem}")));
        }
    }
    lines.push(Line::from(""));
    lines.push(section_title("Overlay"));
    if app.panel_collapsed(crate::app::PanelId::Notifications) {
        lines.push(Line::from("Diagnostics overlay: closed (press n)"));
    } else {
        lines.push(Line::from("Diagnostics overlay: open"));
    }
    let paragraph = Paragraph::new(lines).wrap(Wrap { trim: true });
    frame.render_widget(paragraph, area);
}

fn section_title(label: &str) -> Line {
    Line::from(Span::styled(
        label.to_string(),
        Style::default()
            .fg(Color::LightBlue)
            .add_modifier(Modifier::BOLD),
    ))
}
