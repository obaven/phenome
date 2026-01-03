use ratatui::{
    layout::{Margin, Rect},
    prelude::{Alignment, Frame},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{
        block::Title, Block, Borders, List, ListItem, Paragraph, Scrollbar,
        ScrollbarOrientation, ScrollbarState,
    },
};

use crate::logging::{LogFilter, LOG_INTERVALS_SECS};
use crate::runtime::EventLevel;
use crate::ui::app::App;
use crate::ui::state::HoverPanel;
use crate::ui::util::{format_age, traveling_glow};

pub fn render_log_controls(frame: &mut Frame, area: Rect, app: &mut App) {
    app.ui.log_controls_area = area;
    if app.ui.collapsed_log_controls {
        app.ui.log_menu_pinned = false;
        app.ui.log_menu_mode = None;
        app.ui.log_menu_area = Rect::default();
        app.ui.log_menu_len = 0;
        app.ui.log_menu_hover_index = None;
        app.ui.log_filter_tag_area = Rect::default();
        app.ui.log_stream_tag_area = Rect::default();
        let mut block = Block::default().title("Log Controls").borders(Borders::ALL);
        if app.ui.hover_panel == HoverPanel::Logs {
            block = block.style(Style::default().bg(Color::Rgb(0, 90, 90)));
        }
        frame.render_widget(block, area);
        return;
    }
    let status = if app.ui.log_paused { "paused" } else { "streaming" };
    let filter_tag = format!("[{}]", app.ui.log_config.filter.as_str());
    let stream_tag = format!("[{} {}s]", status, app.ui.log_config.interval.as_secs());
    let line = Line::from(vec![
        Span::raw("Filter "),
        Span::styled(
            &filter_tag,
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::UNDERLINED),
        ),
        Span::raw(" | "),
        Span::raw("Stream "),
        Span::styled(
            &stream_tag,
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::UNDERLINED),
        ),
        Span::raw(" | click tags"),
    ]);
    let panel = Paragraph::new(line)
        .block(Block::default().title("Log Controls").borders(Borders::ALL))
        .alignment(Alignment::Left);
    frame.render_widget(panel, area);

    let inner = area.inner(Margin {
        horizontal: 1,
        vertical: 1,
    });
    let mut cursor = inner.x;
    cursor = cursor.saturating_add("Filter ".len() as u16);
    app.ui.log_filter_tag_area =
        Rect::new(cursor, inner.y, filter_tag.len() as u16, 1);
    cursor = cursor.saturating_add(filter_tag.len() as u16);
    cursor = cursor.saturating_add(" | ".len() as u16);
    cursor = cursor.saturating_add("Stream ".len() as u16);
    app.ui.log_stream_tag_area =
        Rect::new(cursor, inner.y, stream_tag.len() as u16, 1);

    render_log_menu(frame, app);
}

pub fn render_logs(frame: &mut Frame, area: Rect, app: &mut App) {
    app.ui.logs_area = area;
    if app.ui.collapsed_logs {
        let mut block = Block::default()
            .title(format!("Logs ({})", app.ui.log_config.filter.as_str()))
            .borders(Borders::ALL);
        if app.ui.hover_panel == HoverPanel::Logs {
            block = block.style(Style::default().bg(Color::Rgb(0, 90, 90)));
        }
        frame.render_widget(block, area);
        return;
    }
    let events = app.filtered_events();
    let max_items = area.height.saturating_sub(2) as usize;
    let total = events.len();
    let max_offset = total.saturating_sub(max_items);
    let offset = app.ui.log_scroll.min(max_offset as u16) as usize;
    let start = total.saturating_sub(max_items).saturating_sub(offset);

    let items: Vec<ListItem> = events[start..]
        .iter()
        .enumerate()
        .map(|(offset, event)| {
            let level_style = match event.level {
                EventLevel::Info => Style::default().fg(Color::Cyan),
                EventLevel::Warn => Style::default().fg(Color::Yellow),
                EventLevel::Error => Style::default().fg(Color::Red),
            };
            let timestamp = format_age(event.timestamp_ms);
            let line = Line::from(vec![
                Span::styled(format!("[{:<4}]", event.level.as_str()), level_style),
                Span::raw(" "),
                Span::raw(timestamp),
                Span::raw(" "),
                Span::raw(&event.message),
            ]);
            let line_index = start.saturating_add(offset);
            let mut line_style = Style::default();
            if let Some(color) = traveling_glow(line_index, total) {
                line_style = line_style.fg(color);
            }
            ListItem::new(line).style(line_style)
        })
        .collect();

    let updated_secs = app.ui.last_log_emit.elapsed().as_secs();
    let title_left = Title::from(format!("Logs ({})", app.ui.log_config.filter.as_str()))
        .alignment(Alignment::Left);
    let title_right = Title::from(format!(
        "events: {} | updated: {}s",
        total, updated_secs
    ))
    .alignment(Alignment::Right);
    let mut list_block = Block::default()
        .title(title_left)
        .title(title_right)
        .borders(Borders::ALL);
    if app.refresh_pulse_active() {
        list_block = list_block.style(Style::default().fg(Color::Cyan));
    }
    if app.ui.hover_panel == HoverPanel::Logs {
        let active_style = Style::default().bg(Color::Rgb(0, 90, 90));
        list_block = list_block.style(active_style);
    }
    let mut list = List::new(items).block(list_block);
    if app.ui.hover_panel == HoverPanel::Logs {
        let active_style = Style::default().bg(Color::Rgb(0, 90, 90));
        list = list.style(active_style);
    }
    frame.render_widget(list, area);

    if total > max_items && max_items > 0 {
        let mut state = ScrollbarState::new(total).position(app.ui.log_scroll as usize);
        let bar =
            Scrollbar::new(ScrollbarOrientation::VerticalRight).style(Style::default().fg(Color::Cyan));
        frame.render_stateful_widget(
            bar,
            area.inner(Margin {
                horizontal: 0,
                vertical: 1,
            }),
            &mut state,
        );
    }
}

#[derive(Clone)]
struct LogMenuItem {
    label: String,
    selected: bool,
}

fn log_menu_items(app: &App, mode: crate::ui::state::LogMenuMode) -> Vec<LogMenuItem> {
    let filter = app.ui.log_config.filter;
    let interval = app.ui.log_config.interval.as_secs();
    let paused = app.ui.log_paused;
    match mode {
        crate::ui::state::LogMenuMode::Filter => vec![
            LogMenuItem {
                label: "Filter: all".to_string(),
                selected: matches!(filter, LogFilter::All),
            },
            LogMenuItem {
                label: "Filter: info".to_string(),
                selected: matches!(filter, LogFilter::Info),
            },
            LogMenuItem {
                label: "Filter: warn".to_string(),
                selected: matches!(filter, LogFilter::Warn),
            },
            LogMenuItem {
                label: "Filter: error".to_string(),
                selected: matches!(filter, LogFilter::Error),
            },
        ],
        crate::ui::state::LogMenuMode::Stream => {
            let mut items: Vec<LogMenuItem> = LOG_INTERVALS_SECS
                .iter()
                .map(|secs| LogMenuItem {
                    label: format!("Stream: {}s", secs),
                    selected: interval == *secs,
                })
                .collect();
            items.push(LogMenuItem {
                label: if paused {
                    "Stream: resume".to_string()
                } else {
                    "Stream: pause".to_string()
                },
                selected: paused,
            });
            items
        }
    }
}

fn render_log_menu(frame: &mut Frame, app: &mut App) {
    let Some((_x, _y)) = app.ui.mouse_pos else {
        app.ui.log_menu_area = Rect::default();
        app.ui.log_menu_len = 0;
        app.ui.log_menu_hover_index = None;
        return;
    };
    if !app.ui.log_menu_pinned || app.ui.collapsed_log_controls {
        app.ui.log_menu_area = Rect::default();
        app.ui.log_menu_len = 0;
        app.ui.log_menu_hover_index = None;
        return;
    }

    let bounds = if app.ui.body_area.height > 0 {
        app.ui.body_area
    } else {
        app.ui.screen_area
    };
    let Some(mode) = app.ui.log_menu_mode else {
        app.ui.log_menu_area = Rect::default();
        app.ui.log_menu_len = 0;
        app.ui.log_menu_hover_index = None;
        return;
    };
    let items = log_menu_items(app, mode);
    let max_note = items
        .iter()
        .map(|item| item.label.len() as u16)
        .max()
        .unwrap_or(16);
    let width = max_note
        .saturating_add(4)
        .min(app.ui.log_controls_area.width)
        .max(12);
    let height = items.len() as u16 + 2;
    let area = app.ui.log_controls_area;
    let inner = area.inner(Margin {
        horizontal: 1,
        vertical: 1,
    });
    let anchor_y = inner.y.saturating_add(1);
    let screen_bottom = bounds.y.saturating_add(bounds.height);
    let mut x = if mode == crate::ui::state::LogMenuMode::Filter {
        app.ui.log_filter_tag_area.x
    } else {
        app.ui.log_stream_tag_area.x
    };
    let right_bound = bounds.x.saturating_add(bounds.width);
    if x.saturating_add(width) > right_bound {
        x = right_bound.saturating_sub(width);
    }
    let y = if anchor_y.saturating_add(height) <= screen_bottom {
        anchor_y
    } else {
        inner.y.saturating_sub(height)
    };
    let y = y.max(bounds.y);
    let height = height.min(screen_bottom.saturating_sub(y)).max(3);
    let menu_area = Rect::new(x, y, width, height);
    app.ui.log_menu_area = menu_area;
    app.ui.log_menu_len = items.len();

    let list_items: Vec<ListItem> = items
        .iter()
        .enumerate()
        .map(|(index, item)| {
            let mut style = if item.selected {
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            if app.ui.log_menu_hover_index == Some(index) {
                style = style.bg(Color::DarkGray);
            }
            ListItem::new(Line::from(item.label.clone())).style(style)
        })
        .collect();
    let list = List::new(list_items)
        .block(Block::default().title("Log Menu").borders(Borders::ALL));
    frame.render_widget(list, menu_area);
}
