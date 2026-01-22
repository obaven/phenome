use ratatui::layout::Rect;
use ratatui::prelude::Frame;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, List, ListItem};

use crate::app::{App, NavAction, NavSection, nav_items};

pub(super) fn render_flyout(f: &mut Frame, area: Rect, app: &mut App) {
    let body_area = app.ui.body_area;
    if body_area.width == 0 || body_area.height == 0 {
        app.ui.nav_flyout_area = Rect::default();
        app.ui.nav_flyout_count = 0;
        return;
    }
    let items = nav_items(app.active_nav());
    let max_label = items.iter().map(|item| item.label.len()).max().unwrap_or(0);
    let mut flyout_width = (max_label.saturating_add(6)) as u16;
    if flyout_width < 18 {
        flyout_width = 18;
    }
    if flyout_width > 34 {
        flyout_width = 34;
    }
    let available = area.x.saturating_sub(body_area.x);
    if available < 8 {
        app.ui.nav_flyout_area = Rect::default();
        app.ui.nav_flyout_count = 0;
        return;
    }
    if flyout_width > available {
        flyout_width = available;
    }
    let flyout_area = Rect::new(
        area.x.saturating_sub(flyout_width),
        body_area.y,
        flyout_width,
        body_area.height,
    );
    app.ui.nav_flyout_area = flyout_area;
    app.ui.nav_flyout_count = items.len().min(app.ui.nav_flyout_item_areas.len());
    for slot in app.ui.nav_flyout_item_areas.iter_mut() {
        *slot = Rect::default();
    }

    let title = app.active_nav().title();
    let block = Block::default()
        .title(Span::styled(
            title,
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ))
        .style(Style::default().bg(Color::Rgb(20, 22, 26)));

    f.render_widget(
        Block::default().style(Style::default().bg(Color::Rgb(20, 22, 26))),
        flyout_area,
    );

    let inner = block.inner(flyout_area);
    f.render_widget(block, flyout_area);

    let mut list_items = Vec::new();
    for (index, item) in items.iter().enumerate() {
        let marker = if item.action != NavAction::None { " *" } else { "" };
        let label = format!("{}{}", item.label, marker);
        let style = match app.active_nav() {
            NavSection::Terminal if item.action != NavAction::None => Style::default()
                .fg(Color::LightBlue)
                .bg(Color::Rgb(20, 22, 26)),
            _ => Style::default().fg(Color::White).bg(Color::Rgb(20, 22, 26)),
        };
        list_items.push(ListItem::new(Line::from(Span::styled(label, style))));
        if index < app.ui.nav_flyout_item_areas.len() {
            let row = inner.y.saturating_add(index as u16);
            if row < inner.y.saturating_add(inner.height) {
                app.ui.nav_flyout_item_areas[index] = Rect::new(inner.x, row, inner.width, 1);
            }
        }
    }

    let mut list_state = ratatui::widgets::ListState::default();
    if !list_items.is_empty() {
        let selected = app
            .nav_sub_index(app.active_nav())
            .min(list_items.len().saturating_sub(1));
        list_state.select(Some(selected));
    }
    let list = List::new(list_items)
        .highlight_style(
            Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("> ");
    f.render_stateful_widget(list, inner, &mut list_state);
}
