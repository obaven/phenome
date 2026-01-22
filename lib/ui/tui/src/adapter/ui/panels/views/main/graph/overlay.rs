use ratatui::layout::Rect;
use ratatui::prelude::Frame;
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Borders, Paragraph};

use crate::app::App;

pub(super) fn render_search_overlay(frame: &mut Frame, graph_area: Rect, app: &App) {
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
}
