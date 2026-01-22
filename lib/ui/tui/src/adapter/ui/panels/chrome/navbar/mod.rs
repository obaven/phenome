use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::prelude::Frame;
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Borders};

use crate::app::App;

mod flyout;
mod items;

pub struct NavbarPanel;

impl NavbarPanel {
    pub fn new() -> Self {
        Self
    }

    pub fn render(&self, f: &mut Frame, area: Rect, app: &mut App) {
        let active_index = app.active_nav().index();
        let block = Block::default()
            .borders(Borders::LEFT)
            .border_style(Style::default().fg(Color::DarkGray))
            .style(Style::default().bg(Color::Rgb(16, 18, 22)));
        f.render_widget(block, area);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([
                Constraint::Length(5),
                Constraint::Length(5),
                Constraint::Length(5),
                Constraint::Min(0),
            ])
            .split(area);

        for (index, (item, chunk)) in items::NAV_ITEMS.iter().zip(chunks.iter()).enumerate() {
            if index < app.ui.navbar_item_areas.len() {
                app.ui.navbar_item_areas[index] = *chunk;
            }
            items::render_item(f, *chunk, item, index == active_index);
        }

        flyout::render_flyout(f, area, app);
    }
}

pub fn render_navbar(f: &mut Frame, area: Rect, app: &mut App) {
    let panel = NavbarPanel::new();
    panel.render(f, area, app);
}
