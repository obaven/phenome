use bootstrappo::application::timing::{TimingComparison, TimingHistory};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Row, Table},
};
use rotappo_ports::{AccessUrlInfo, ComponentState, ComponentStatus, PortSet};
use std::collections::HashMap;
use std::time::Duration;

pub struct SuccessSummaryPanel {}

struct SummaryData {
    total_duration: Duration,
    completed: usize,
    deferred: usize,
    failed: usize,
    total: usize,
    success_rate: f32,
    #[allow(dead_code)]
    comparison: Option<TimingComparison>,
    render_duration: Duration,
    apply_duration: Duration,
    wait_duration: Duration,
    render_percentage: f32,
    apply_percentage: f32,
    wait_percentage: f32,
    access_urls: Vec<AccessUrlInfo>,
    hotspots: Vec<HotspotInfo>,
}

struct HotspotInfo {
    component: String,
    total_time: Duration,
    wait_time: Duration,
    retry_count: u32,
}

impl SuccessSummaryPanel {
    pub fn new() -> Self {
        Self {}
    }

    pub fn render(&mut self, f: &mut Frame, area: Rect, ports: &PortSet) {
        let bootstrap_port = ports.bootstrap.clone();
        let states = bootstrap_port.component_states();
        let timing_history = bootstrap_port.timing_history();

        let access_urls = bootstrap_port.access_urls();
        let summary = self.calculate_summary(&states, &timing_history, access_urls);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(8),  // Overall status
                Constraint::Length(8),  // Timing breakdown
                Constraint::Length(10), // Access URLs
                Constraint::Length(8),  // Top hotspots
                Constraint::Min(0),     // Remaining space
            ])
            .split(area);

        self.render_overall_status(f, chunks[0], &summary);
        self.render_timing_breakdown(f, chunks[1], &summary);
        self.render_access_urls(f, chunks[2], &summary);
        self.render_hotspots(f, chunks[3], &summary);
    }

    fn calculate_summary(
        &self,
        states: &HashMap<String, ComponentState>,
        history: &Option<TimingHistory>,
        access_urls: Vec<AccessUrlInfo>,
    ) -> SummaryData {
        let total = states.len();
        let completed = states
            .values()
            .filter(|s| matches!(s.status, ComponentStatus::Complete))
            .count();
        let deferred = states
            .values()
            .filter(|s| matches!(s.status, ComponentStatus::Deferred))
            .count();
        let failed = states
            .values()
            .filter(|s| matches!(s.status, ComponentStatus::Failed))
            .count();

        let success_rate = if total > 0 {
            completed as f32 / total as f32
        } else {
            0.0
        };

        // Mocked breakdown - in real impl, this would sum up actual detailed timings if available
        let render_duration = Duration::from_secs(18);
        let apply_duration = Duration::from_secs(42);
        let wait_duration = Duration::from_secs(107);
        let total_duration = render_duration + apply_duration + wait_duration; // This likely differs from wall clock

        let render_percentage =
            (render_duration.as_secs_f32() / total_duration.as_secs_f32()) * 100.0;
        let apply_percentage =
            (apply_duration.as_secs_f32() / total_duration.as_secs_f32()) * 100.0;
        let wait_percentage = (wait_duration.as_secs_f32() / total_duration.as_secs_f32()) * 100.0;

        // Comparison logic
        let comparison = if let Some(_) = history {
            // In a real scenario, we'd form a "current" entry.
            // For now, let's assume the last entry IS current if we just finished.
            // Or we just return None if we can't properly construct it here without more context.
            None
        } else {
            None
        };

        // Mock Hotspots
        let hotspots = vec![
            HotspotInfo {
                component: "authelia".into(),
                total_time: Duration::from_secs(38),
                wait_time: Duration::from_secs(32),
                retry_count: 0,
            },
            HotspotInfo {
                component: "velero".into(),
                total_time: Duration::from_secs(31),
                wait_time: Duration::from_secs(28),
                retry_count: 0,
            },
        ];

        SummaryData {
            total_duration,
            completed,
            deferred,
            failed,
            total,
            success_rate,
            comparison,
            render_duration,
            apply_duration,
            wait_duration,
            render_percentage,
            apply_percentage,
            wait_percentage,
            access_urls,
            hotspots,
        }
    }

    fn render_overall_status(&self, f: &mut Frame, area: Rect, summary: &SummaryData) {
        let text = vec![
            Line::from(vec![Span::styled(
                "🎉 Bootstrap Complete!",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from(""),
            Line::from(vec![
                Span::raw("Total Time: "),
                Span::styled(
                    format!("{:?}", summary.total_duration),
                    Style::default().fg(Color::Cyan),
                ),
            ]),
            Line::from(vec![
                Span::raw("Complete: "),
                Span::styled(
                    format!("{}/{}", summary.completed, summary.total),
                    Style::default().fg(Color::Green),
                ),
                Span::raw("  Deferred: "),
                Span::styled(
                    format!("{}", summary.deferred),
                    Style::default().fg(Color::Yellow),
                ),
                Span::raw("  Failed: "),
                Span::styled(
                    format!("{}", summary.failed),
                    Style::default().fg(Color::Red),
                ),
            ]),
            Line::from(vec![
                Span::raw("Success Rate: "),
                Span::styled(
                    format!("{:.1}%", summary.success_rate * 100.0),
                    Style::default().fg(Color::Green),
                ),
            ]),
        ];

        let paragraph = Paragraph::new(text).block(
            Block::default()
                .title("Overall Status")
                .borders(Borders::ALL),
        );

        f.render_widget(paragraph, area);
    }

    fn render_timing_breakdown(&self, f: &mut Frame, area: Rect, summary: &SummaryData) {
        let rows = vec![
            Row::new(vec![
                "Phase".to_string(),
                "Time".to_string(),
                "% of Total".to_string(),
                "vs Previous".to_string(),
            ]),
            Row::new(vec![
                "Render".to_string(),
                format!("{:?}", summary.render_duration),
                format!("{:.1}%", summary.render_percentage),
                "-".to_string(), // Delta placeholder
            ]),
            Row::new(vec![
                "Apply".to_string(),
                format!("{:?}", summary.apply_duration),
                format!("{:.1}%", summary.apply_percentage),
                "-".to_string(), // Delta placeholder
            ]),
            Row::new(vec![
                "Wait".to_string(),
                format!("{:?}", summary.wait_duration),
                format!("{:.1}%", summary.wait_percentage),
                "-".to_string(), // Delta placeholder
            ]),
        ];

        let table = Table::new(
            rows,
            [
                Constraint::Percentage(25),
                Constraint::Percentage(25),
                Constraint::Percentage(25),
                Constraint::Percentage(25),
            ],
        )
        .block(
            Block::default()
                .title("Timing Breakdown")
                .borders(Borders::ALL),
        );

        f.render_widget(table, area);
    }

    fn render_access_urls(&self, f: &mut Frame, area: Rect, summary: &SummaryData) {
        let rows = summary
            .access_urls
            .iter()
            .map(|url_info| {
                Row::new(vec![
                    url_info.service.clone(),
                    url_info.url.clone(),
                    url_info.status.label().to_string(),
                ])
            })
            .collect::<Vec<_>>();

        let table = Table::new(
            rows,
            [
                Constraint::Percentage(30),
                Constraint::Percentage(50),
                Constraint::Percentage(20),
            ],
        )
        .header(Row::new(vec!["Service", "URL", "Status"]))
        .block(Block::default().title("Access URLs").borders(Borders::ALL));

        f.render_widget(table, area);
    }

    fn render_hotspots(&self, f: &mut Frame, area: Rect, summary: &SummaryData) {
        let rows = summary
            .hotspots
            .iter()
            .map(|hotspot| {
                Row::new(vec![
                    hotspot.component.clone(),
                    format!("{:?}", hotspot.total_time),
                    format!("{:?}", hotspot.wait_time),
                    format!("{}", hotspot.retry_count),
                ])
            })
            .collect::<Vec<_>>();

        let table = Table::new(
            rows,
            [
                Constraint::Percentage(40),
                Constraint::Percentage(20),
                Constraint::Percentage(20),
                Constraint::Percentage(20),
            ],
        )
        .header(Row::new(vec![
            "Component",
            "Total Time",
            "Wait Time",
            "Retries",
        ]))
        .block(
            Block::default()
                .title("Top 5 Hotspots")
                .borders(Borders::ALL),
        );

        f.render_widget(table, area);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rotappo_ports::ComponentStatus;
    use std::collections::HashMap;

    #[test]
    fn test_calculate_summary() {
        let mut states = HashMap::new();
        states.insert(
            "comp1".to_string(),
            ComponentState {
                id: "comp1".to_string(),
                status: ComponentStatus::Complete,
                readiness: None,
                timing: Default::default(),
                retry_count: 0,
                deferred_reason: None,
            },
        );
        states.insert(
            "comp2".to_string(),
            ComponentState {
                id: "comp2".to_string(),
                status: ComponentStatus::Deferred,
                readiness: None,
                timing: Default::default(),
                retry_count: 0,
                deferred_reason: None,
            },
        );
        let panel = SuccessSummaryPanel::new();
        let summary = panel.calculate_summary(&states, &None, vec![]);

        assert_eq!(summary.total, 2);
        assert_eq!(summary.completed, 1);
        assert_eq!(summary.deferred, 1);
        assert_eq!(summary.failed, 0);
        assert_eq!(summary.success_rate, 0.5);
    }
}
