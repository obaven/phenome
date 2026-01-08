use ratatui::{
    layout::{Margin, Rect},
    prelude::Frame,
    style::{Color, Style},
    text::Line,
    widgets::{List, ListItem, Scrollbar, ScrollbarOrientation, ScrollbarState},
};

use crate::app::App;
use crate::state::HoverPanel;
use crate::util::{collect_problems, traveling_glow};

/// Render the problems panel.
///
/// # Examples
/// ```rust,no_run
/// use ratatui::{backend::TestBackend, Terminal};
/// use rotappo_ui_tui::app::App;
/// use rotappo_ui_tui::panels::render_problems;
///
/// # fn app() -> App { todo!() }
/// let backend = TestBackend::new(80, 24);
/// let mut terminal = Terminal::new(backend).unwrap();
/// let mut app = app();
/// terminal
///     .draw(|frame| render_problems(frame, frame.area(), &mut app))
///     .unwrap();
/// ```
pub fn render_problems(frame: &mut Frame, area: Rect, app: &mut App) {
    app.ui.problems_area = area;
    let hovered = app.ui.hover_panel == HoverPanel::Problems;
    if app.ui.collapsed_problems {
        let block = crate::ui_panel_block!("Problems", hovered);
        frame.render_widget(block, area);
        return;
    }
    let problems = collect_problems(app);
    let total_problems = problems.len();
    let view_height = area.height.saturating_sub(2) as usize;
    let offset = app.ui.problems_scroll as usize;
    let items: Vec<ListItem> = if problems.is_empty() {
        vec![ListItem::new(Line::from("No problems detected"))]
    } else {
        problems
            .into_iter()
            .enumerate()
            .skip(offset)
            .take(view_height.max(1))
            .map(|(index, problem)| {
                let mut line_style = Style::default();
                if let Some(color) = traveling_glow(index, total_problems) {
                    line_style = line_style.fg(color);
                }
                let mut item = ListItem::new(Line::from(problem)).style(line_style);
                if app.ui.hover_problem_index == Some(index) {
                    item = item.style(Style::default().bg(Color::DarkGray));
                }
                item
            })
            .collect()
    };
    let list_block = crate::ui_panel_block!("Problems", hovered, app.refresh_pulse_active());
    let mut list = List::new(items).block(list_block);
    if hovered {
        let active_style = Style::default().bg(Color::Rgb(0, 90, 90));
        list = list.style(active_style);
    }
    frame.render_widget(list, area);

    if total_problems > view_height && view_height > 0 {
        let mut state =
            ScrollbarState::new(total_problems).position(app.ui.problems_scroll as usize);
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
