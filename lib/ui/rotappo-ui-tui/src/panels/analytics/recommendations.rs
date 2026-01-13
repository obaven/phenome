use ratatui::{
    layout::{Constraint, Rect},
    prelude::Frame,
    style::{Color, Modifier, Style, Stylize},
    widgets::{Block, Borders, Cell, Padding, Row, Table},
};

use crate::app::App;
use crate::util::centered_rect;
use rotappo_domain::{Priority, RecommendationAction, RecommendationStatus};

pub fn render_recommendations(frame: &mut Frame, area: Rect, app: &mut App) {
    let recommendations = app
        .analytics_recommendations
        .as_ref()
        .map(|recs| recs.as_slice())
        .unwrap_or_default();

    let block = Block::default()
        .borders(Borders::ALL)
        .title("Recommendations")
        .padding(Padding::uniform(1));

    let inner_area = block.inner(area);
    frame.render_widget(block, area);

    if recommendations.is_empty() {
        let msg = if app.analytics_recommendations.is_none() {
            "Waiting for data..."
        } else {
            "No active recommendations."
        };
        frame.render_widget(
            ratatui::widgets::Paragraph::new(msg)
                .style(Style::default().fg(Color::DarkGray).italic())
                .alignment(ratatui::layout::Alignment::Center),
            centered_rect(50, 50, area),
        );
        return;
    }

    let rows: Vec<Row> = recommendations
        .iter()
        .map(|rec| {
            let (priority_style, priority_label) = match rec.priority {
                Priority::High => (
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                    "HIGH",
                ),
                Priority::Medium => (Style::default().fg(Color::Yellow), "MED"),
                Priority::Low => (Style::default().fg(Color::Green), "LOW"),
            };

            let action_str = match &rec.action {
                RecommendationAction::ScaleDeployment { name, from, to } => {
                    format!("Scale {} {}->{}", name, from, to)
                }
                RecommendationAction::UpdateResourceLimits { resource, .. } => {
                    format!("Limit {}", resource)
                }
                RecommendationAction::ReclaimStorage { volume, size_gb } => {
                    format!("Reclaim {} {}GB", volume, size_gb)
                }
            };

            let (status_style, status_label) = match rec.status {
                RecommendationStatus::Applied { .. } => {
                    (Style::default().fg(Color::Green), "APPLIED")
                }
                RecommendationStatus::Dismissed { .. } => {
                    (Style::default().fg(Color::DarkGray), "DISMISSED")
                }
                RecommendationStatus::Scheduled { .. } => {
                    (Style::default().fg(Color::Cyan), "SCHEDULED")
                }
                RecommendationStatus::Pending => (Style::default().fg(Color::Yellow), "PENDING"),
            };

            Row::new(vec![
                Cell::from(rec.title.clone()),
                Cell::from(priority_label).style(priority_style),
                Cell::from(action_str),
                Cell::from(status_label).style(status_style),
            ])
            .height(1)
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Percentage(40), // Title
            Constraint::Percentage(15), // Priority
            Constraint::Percentage(30), // Action
            Constraint::Percentage(15), // Status
        ],
    )
    .header(
        Row::new(vec!["Title", "Priority", "Action", "Status"]).style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD)
                .underlined(),
        ),
    )
    .row_highlight_style(Style::default().add_modifier(Modifier::BOLD));

    frame.render_widget(table, inner_area);
}
