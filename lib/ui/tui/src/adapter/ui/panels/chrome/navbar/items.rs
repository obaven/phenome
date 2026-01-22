use ratatui::layout::{Alignment, Rect};
use ratatui::prelude::Frame;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

pub(super) struct NavItem {
    pub(super) icon: &'static str,
    pub(super) label: &'static str,
}

pub(super) const NAV_ITEMS: [NavItem; 3] = [
    NavItem {
        icon: "📊",
        label: "Analytics",
    },
    NavItem {
        icon: "🕸️",
        label: "Topology",
    },
    NavItem {
        icon: "💻",
        label: "Terminal",
    },
];

pub(super) fn render_item(f: &mut Frame, area: Rect, item: &NavItem, active: bool) {
    let icon_style = if active {
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Gray)
    };
    let label_style = if active {
        Style::default()
            .fg(Color::White)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let text = vec![
        Line::from(Span::styled(item.icon, icon_style)),
        Line::from(Span::styled(item.label, label_style)),
    ];
    let mut block = Block::default().borders(Borders::NONE);
    if active {
        block = block.style(Style::default().bg(Color::Rgb(0, 70, 80)));
    }
    let paragraph = Paragraph::new(text)
        .block(block)
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true });
    f.render_widget(paragraph, area);
}
