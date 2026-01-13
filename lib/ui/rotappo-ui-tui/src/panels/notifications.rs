use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
};

#[derive(Clone, Copy)]
struct AlertItem {
    source: &'static str,
    message: &'static str,
    hint: &'static str,
    severity: AlertSeverity,
}

#[derive(Clone, Copy)]
enum AlertSeverity {
    Critical,
    Warning,
}

impl AlertSeverity {
    fn label(self) -> &'static str {
        match self {
            AlertSeverity::Critical => "CRIT",
            AlertSeverity::Warning => "WARN",
        }
    }

    fn style(self) -> Style {
        match self {
            AlertSeverity::Critical => Style::default()
                .fg(Color::Red)
                .add_modifier(Modifier::BOLD),
            AlertSeverity::Warning => Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        }
    }
}

const ALERTS: [AlertItem; 2] = [
    AlertItem {
        source: "backend-api",
        message: "High memory usage detected (92% of limit)",
        hint: "Consider increasing memory limits",
        severity: AlertSeverity::Warning,
    },
    AlertItem {
        source: "worker-job",
        message: "ImagePullBackOff: Failed to pull image",
        hint: "Verify image repository credentials",
        severity: AlertSeverity::Critical,
    },
];

pub struct NotificationPanel {}

impl NotificationPanel {
    pub fn new() -> Self {
        Self {}
    }

    pub fn render(&self, f: &mut Frame, area: Rect) {
        if area.width == 0 || area.height == 0 {
            return;
        }
        let block = Block::default()
            .title(Span::styled(
                "Error Diagnostics (n to close)",
                Style::default().add_modifier(Modifier::BOLD),
            ))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray))
            .style(Style::default().bg(Color::Rgb(22, 24, 28)));

        let inner = block.inner(area);
        f.render_widget(block, area);

        let mut lines = Vec::new();
        for (index, item) in ALERTS.iter().enumerate() {
            lines.push(Line::from(vec![
                Span::styled(item.severity.label(), item.severity.style()),
                Span::raw(" "),
                Span::styled(
                    item.source,
                    Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
                ),
            ]));
            lines.push(Line::from(Span::styled(
                item.message,
                Style::default().fg(Color::Gray),
            )));
            lines.push(Line::from(vec![
                Span::styled("Hint: ", Style::default().fg(Color::Cyan)),
                Span::styled(item.hint, Style::default().fg(Color::LightBlue)),
            ]));
            if index + 1 < ALERTS.len() {
                lines.push(Line::from(""));
            }
        }

        let paragraph = Paragraph::new(lines).wrap(Wrap { trim: true });
        f.render_widget(paragraph, inner);
    }
}

pub fn render_notifications(f: &mut Frame, area: Rect) {
    let panel = NotificationPanel::new();
    panel.render(f, area);
}
