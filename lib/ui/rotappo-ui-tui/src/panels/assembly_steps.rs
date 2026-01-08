use ratatui::{
    layout::{Margin, Rect},
    prelude::Frame,
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, Scrollbar, ScrollbarOrientation, ScrollbarState},
};

use crate::app::App;
use crate::state::HoverPanel;
use crate::util::{assembly_lines, traveling_glow};

/// Render the assembly steps panel.
///
/// # Examples
/// ```rust,no_run
/// use ratatui::{backend::TestBackend, Terminal};
/// use rotappo_ui_tui::app::App;
/// use rotappo_ui_tui::panels::render_assembly_steps;
///
/// # fn app() -> App { todo!() }
/// let backend = TestBackend::new(80, 24);
/// let mut terminal = Terminal::new(backend).unwrap();
/// let mut app = app();
/// terminal
///     .draw(|frame| render_assembly_steps(frame, frame.area(), &mut app))
///     .unwrap();
/// ```
pub fn render_assembly_steps(frame: &mut Frame, area: Rect, app: &mut App) {
    app.ui.assembly_area = area;
    if app.ui.collapsed_assembly_steps {
        let mut block = Block::default().title("Assembly Steps").borders(Borders::ALL);
        if app.ui.hover_panel == HoverPanel::Assembly {
            block = block.style(Style::default().bg(Color::Rgb(0, 90, 90)));
        }
        frame.render_widget(block, area);
        return;
    }
    let snapshot = app.runtime.snapshot();
    let items: Vec<ListItem> = if snapshot.assembly_steps.is_empty() {
        vec![ListItem::new("No assembly steps loaded")]
    } else {
        let lines = assembly_lines(snapshot);
        let total_lines = lines.len();
        lines
            .into_iter()
            .skip(app.ui.assembly_scroll as usize)
            .enumerate()
            .map(|(offset, line)| {
                let line_index = app.ui.assembly_scroll as usize + offset;
                let mut line_style = Style::default();
                if let Some(color) = traveling_glow(line_index, total_lines) {
                    line_style = line_style.fg(color);
                }
                let mut item = ListItem::new(line.line).style(line_style);
                if let Some(step_index) = line.step_index {
                    if app.ui.hover_action_index == Some(step_index) {
                        item = item.style(Style::default().bg(Color::DarkGray));
                    }
                }
                item
            })
            .collect()
    };

    let mut list_block = Block::default().title("Assembly Steps").borders(Borders::ALL);
    if app.refresh_pulse_active() {
        list_block = list_block.style(Style::default().fg(Color::Cyan));
    }
        if app.ui.hover_panel == HoverPanel::Assembly {
            let active_style = Style::default().bg(Color::Rgb(0, 90, 90));
            list_block = list_block.style(active_style);
        }
    let mut list = List::new(items)
        .block(list_block)
        .highlight_style(Style::default().add_modifier(Modifier::BOLD));
    if app.ui.hover_panel == HoverPanel::Assembly {
        let active_style = Style::default().bg(Color::Rgb(0, 90, 90));
        list = list.style(active_style);
    }
    frame.render_widget(list, area);

    let total_lines = assembly_lines(snapshot).len();
    let view_height = area.height.saturating_sub(2) as usize;
    if total_lines > view_height && view_height > 0 {
        let mut state =
            ScrollbarState::new(total_lines).position(app.ui.assembly_scroll as usize);
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
