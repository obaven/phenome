use ratatui::{
    layout::Rect,
    prelude::{Alignment, Frame},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
};

use crate::app::App;

/// Render the footer help panel.
///
/// # Examples
/// ```rust,no_run
/// use ratatui::{backend::TestBackend, Terminal};
/// use rotappo_ui_tui::app::App;
/// use rotappo_ui_tui::panels::render_footer;
///
/// # fn app() -> App { todo!() }
/// let backend = TestBackend::new(80, 24);
/// let mut terminal = Terminal::new(backend).unwrap();
/// let mut app = app();
/// terminal
///     .draw(|frame| render_footer(frame, frame.area(), &mut app))
///     .unwrap();
/// ```
pub fn render_footer(frame: &mut Frame, area: Rect, app: &mut App) {
    app.ui.help_area = area;
    if app.ui.collapsed_help {
        let block = Block::default().title("Help").borders(Borders::ALL);
        frame.render_widget(block, area);
        return;
    }
    let lines = help_lines(app);
    let paragraph = Paragraph::new(lines)
        .block(Block::default().title("Help").borders(Borders::ALL))
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: true });
    frame.render_widget(paragraph, area);
}

fn help_lines(app: &App) -> Vec<Line> {
    let mut lines = Vec::new();
    lines.push(section_title("Navigation"));
    lines.push(Line::from("1/2/3: switch section  a: analytics  tab/shift+tab: cycle sections"));
    lines.push(Line::from("left/right: cycle sections  [ ]: cycle menu"));
    lines.push(Line::from("enter: activate menu  n: toggle diagnostics"));
    lines.push(Line::from("menu items with * run a command"));
    lines.push(Line::from("q/esc: quit  r: refresh snapshot"));
    if let Some(item) = app.active_subitem() {
        lines.push(Line::from(format!(
            "Active: {} > {}",
            app.active_nav().title(),
            item.label
        )));
    }
    lines.push(Line::from(""));

    match app.active_view() {
        crate::app::NavView::AnalyticsRealtime
        | crate::app::NavView::AnalyticsHistorical
        | crate::app::NavView::AnalyticsPredictions
        | crate::app::NavView::AnalyticsRecommendations
        | crate::app::NavView::AnalyticsInsights => {
            lines.push(section_title("Analytics"));
            lines.push(Line::from("1-4: switch analytics views"));
        }
        crate::app::NavView::TopologyAssembly
        | crate::app::NavView::TopologyDomains
        | crate::app::NavView::TopologyCapabilities
        | crate::app::NavView::TopologyQueue
        | crate::app::NavView::TopologyHealth
        | crate::app::NavView::TopologyDagGraph
        | crate::app::NavView::TopologyDualGraph => {
            lines.push(section_title("Topology"));
            lines.push(Line::from("click: select node  enter: activate"));
            lines.push(Line::from("arrows: navigate  shift+arrows: pan"));
            lines.push(Line::from("+/-: zoom  0: reset view"));
            lines.push(Line::from("paths highlight dependencies from selection"));
            if let Some(node) = app.graph.selected_node() {
                lines.push(Line::from(format!("Selected: {}", node.label)));
            }
        }
        crate::app::NavView::TerminalLogs | crate::app::NavView::TerminalEvents => {
            lines.push(section_title("Terminal"));
            lines.push(Line::from(format!(
                "f: filter logs (current: {})",
                app.ui.log_config.filter.as_str()
            )));
            lines.push(Line::from("mouse wheel: scroll logs"));
        }
        crate::app::NavView::TerminalCommands => {
            lines.push(section_title("Terminal Commands"));
            lines.push(Line::from("up/down or j/k: move selection"));
            lines.push(Line::from("enter: run selected action"));
        }
        crate::app::NavView::TerminalDiagnostics => {
            lines.push(section_title("Diagnostics"));
            lines.push(Line::from("n: toggle diagnostics overlay"));
        }
    }

    lines.push(Line::from(""));
    lines.push(section_title("System"));
    lines.push(Line::from("y/n/enter: confirm or cancel action"));
    lines.push(Line::from("p (hold 3s): pause stream + pin tooltip"));
    lines.push(Line::from("u (hold 3s): unpin tooltip"));
    lines
}

fn section_title(label: &'static str) -> Line<'static> {
    Line::from(Span::styled(
        label,
        Style::default().add_modifier(Modifier::BOLD),
    ))
}
