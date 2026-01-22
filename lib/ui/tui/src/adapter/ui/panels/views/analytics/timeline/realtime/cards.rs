use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::prelude::Frame;
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, Padding, Paragraph};

pub(super) fn render_stat_card(
    frame: &mut Frame,
    area: Rect,
    title: &str,
    value: &str,
    color: Color,
) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(color))
        .title(title)
        .padding(Padding::uniform(1));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let text = Paragraph::new(value)
        .style(Style::default().fg(color).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center);

    let v_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(1),
            Constraint::Length(1),
            Constraint::Min(1),
        ])
        .split(inner);

    frame.render_widget(text, v_chunks[1]);
}
