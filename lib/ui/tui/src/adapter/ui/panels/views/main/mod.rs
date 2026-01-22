use ratatui::{
    layout::Rect,
    prelude::Frame,
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, Borders},
};

use crate::app::{App, NavView};
use crate::panels::analytics;
use primer::application::flows::reconcile::visualize;

mod graph;
mod shared;
mod terminal;
mod topology;

use graph::render_topology_graph;
use terminal::{
    render_terminal_commands, render_terminal_diagnostics, render_terminal_events,
    render_terminal_logs,
};
use shared::reset_panel_areas;
use topology::{
    render_topology_assembly, render_topology_capabilities, render_topology_domains,
    render_topology_health, render_topology_queue,
};

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
