use ratatui::{
    layout::{Margin, Rect},
    prelude::Frame,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{List, ListItem, Scrollbar, ScrollbarOrientation, ScrollbarState},
};

use crate::app::App;
use crate::state::HoverPanel;
use crate::util::traveling_glow;

/// Render the actions panel.
///
/// # Examples
/// ```rust,no_run
/// use ratatui::{backend::TestBackend, Terminal};
/// use rotappo_ui_tui::app::App;
/// use rotappo_ui_tui::panels::render_actions;
///
/// # fn app() -> App { todo!() }
/// let backend = TestBackend::new(80, 24);
/// let mut terminal = Terminal::new(backend).unwrap();
/// let mut app = app();
/// terminal
///     .draw(|frame| render_actions(frame, frame.area(), &mut app))
///     .unwrap();
/// ```
pub fn render_actions(frame: &mut Frame, area: Rect, app: &mut App) {
    app.ui.actions_area = area;
    let hovered = app.ui.hover_panel == HoverPanel::Actions;
    if app.ui.collapsed_actions {
        let block = crate::ui_panel_block!("Actions", hovered);
        frame.render_widget(block, area);
        return;
    }
    let actions = app.runtime.registry().actions();
    let total_actions = actions.len();
    let view_height = area.height.saturating_sub(2) as usize;
    let visible_items = (view_height / 2).max(1);
    let offset = app.ui.actions_scroll as usize;
    let items: Vec<ListItem> = actions
        .iter()
        .enumerate()
        .skip(offset)
        .take(visible_items)
        .map(|(index, action)| {
            let id_style = if app.action_flash_active(index) {
                Style::default()
                    .fg(Color::LightBlue)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD)
            };
            let content = vec![
                Line::from(vec![
                    Span::styled(action.id.to_string(), id_style),
                    Span::raw(" "),
                    Span::raw(action.label),
                ]),
                Line::from(vec![
                    Span::raw("  "),
                    Span::styled(
                        action.safety.as_str(),
                        Style::default().fg(Color::DarkGray),
                    ),
                    Span::raw(" - "),
                    Span::raw(action.description),
                ]),
            ];
            let mut line_style = Style::default();
            if let Some(color) = traveling_glow(index, total_actions) {
                line_style = line_style.fg(color);
            }
            let mut item = ListItem::new(content).style(line_style);
            if app.ui.hover_action_index == Some(index) {
                item = item.style(Style::default().bg(Color::DarkGray));
            }
            item
        })
        .collect();

    let list_block = crate::ui_panel_block!("Actions", hovered, app.refresh_pulse_active());
    let mut list = List::new(items)
        .block(list_block)
        .highlight_symbol("> ")
        .highlight_style(
            Style::default()
                .fg(Color::Black)
                .bg(Color::White)
                .add_modifier(Modifier::BOLD),
        );
    if hovered {
        let active_style = Style::default().bg(Color::Rgb(0, 90, 90));
        list = list.style(active_style);
    }
    let selected = app.action_state.selected();
    let mut state = ratatui::widgets::ListState::default();
    if let Some(selected) = selected {
        if selected >= offset && selected < offset + visible_items {
            state.select(Some(selected - offset));
        }
    }
    frame.render_stateful_widget(list, area, &mut state);

    if total_actions > visible_items && visible_items > 0 {
        let mut state =
            ScrollbarState::new(total_actions).position(app.ui.actions_scroll as usize);
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
