use ratatui::layout::{Margin, Rect};
use ratatui::prelude::Frame;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{List, ListItem, Scrollbar, ScrollbarOrientation, ScrollbarState};

use crate::app::App;
use phenome_domain::ActionSafety;

pub fn render_terminal_commands(frame: &mut Frame, area: Rect, app: &mut App) {
    render_action_list(frame, area, app);
}

fn render_action_list(frame: &mut Frame, area: Rect, app: &mut App) {
    app.ui.actions_area = area;
    app.ui.collapsed_actions = false;
    let actions = app.runtime.registry().actions();
    let total_actions = actions.len();
    let view_height = area.height.max(1) as usize;
    let visible_items = view_height.max(1);
    let max_offset = total_actions.saturating_sub(visible_items);
    if app.ui.actions_scroll as usize > max_offset {
        app.ui.actions_scroll = max_offset as u16;
    }
    let offset = app.ui.actions_scroll as usize;
    let mut items = Vec::new();
    for action in actions.iter().skip(offset).take(visible_items) {
        let safety_style = match action.safety {
            ActionSafety::Safe => Style::default().fg(Color::Green),
            ActionSafety::Guarded => Style::default().fg(Color::Yellow),
            ActionSafety::Destructive => Style::default().fg(Color::Red),
        };
        let line = Line::from(vec![
            Span::styled(action.id.to_string(), Style::default().fg(Color::Cyan)),
            Span::raw(" "),
            Span::raw(action.label),
            Span::raw(" "),
            Span::styled(format!("[{}]", action.safety.as_str()), safety_style),
        ]);
        items.push(ListItem::new(line));
    }
    let mut list_state = ratatui::widgets::ListState::default();
    if let Some(selected) = app.action_state.selected() {
        if selected >= offset && selected < offset + visible_items {
            list_state.select(Some(selected - offset));
        }
    } else if !items.is_empty() {
        app.action_state.select(Some(0));
        list_state.select(Some(0));
    }
    let list = List::new(items).highlight_symbol("> ").highlight_style(
        Style::default()
            .fg(Color::Black)
            .bg(Color::White)
            .add_modifier(Modifier::BOLD),
    );
    frame.render_stateful_widget(list, area, &mut list_state);

    if total_actions > visible_items && visible_items > 0 {
        let mut state = ScrollbarState::new(total_actions).position(app.ui.actions_scroll as usize);
        let bar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .style(Style::default().fg(Color::Cyan));
        frame.render_stateful_widget(
            bar,
            area.inner(Margin {
                horizontal: 0,
                vertical: 0,
            }),
            &mut state,
        );
    }
}
