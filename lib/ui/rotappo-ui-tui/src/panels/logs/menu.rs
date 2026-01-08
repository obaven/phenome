//! Log menu rendering and data helpers.

use ratatui::{
    layout::{Margin, Rect},
    prelude::Frame,
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Block, Borders, List, ListItem},
};

use rotappo_ui_presentation::logging::{LogFilter, LOG_INTERVALS_SECS};
use crate::app::App;

#[derive(Clone)]
struct LogMenuItem {
    label: String,
    selected: bool,
}

fn log_menu_items(app: &App, mode: crate::state::LogMenuMode) -> Vec<LogMenuItem> {
    let filter = app.ui.log_config.filter;
    let interval = app.ui.log_config.interval.as_secs();
    let paused = app.ui.log_paused;
    match mode {
        crate::state::LogMenuMode::Filter => vec![
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
        crate::state::LogMenuMode::Stream => {
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

pub(super) fn render_log_menu(frame: &mut Frame, app: &mut App) {
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
    let mut x = if mode == crate::state::LogMenuMode::Filter {
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
