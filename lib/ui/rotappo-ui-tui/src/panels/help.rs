use ratatui::{
    layout::Rect,
    prelude::{Alignment, Frame},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
};

use crate::app::App;

/// Render the footer help panel.
///
/// # Examples
/// ```rust,no_run
/// use ratatui::{backend::TestBackend, Terminal};
/// use rotappo_ui_tui::app::App;
/// use rotappo_ui_tui::panels::render_footer;
///
/// # fn app() -> App { todo!() }
/// let backend = TestBackend::new(80, 24);
/// let mut terminal = Terminal::new(backend).unwrap();
/// let mut app = app();
/// terminal
///     .draw(|frame| render_footer(frame, frame.area(), &mut app))
///     .unwrap();
/// ```
pub fn render_footer(frame: &mut Frame, area: Rect, app: &mut App) {
    app.ui.help_area = area;
    if app.ui.collapsed_help {
        let block = Block::default().title("Help").borders(Borders::ALL);
        frame.render_widget(block, area);
        return;
    }
    let lines = help_lines(app);
    let paragraph = Paragraph::new(lines)
        .block(Block::default().title("Help").borders(Borders::ALL))
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: true });
    frame.render_widget(paragraph, area);
}

fn help_lines(app: &App) -> Vec<Line> {
    vec![
        Line::from(Span::styled("Core", Style::default().add_modifier(Modifier::BOLD))),
        Line::from("q/esc: quit  r: refresh snapshot"),
        Line::from("y/n/enter: confirm or cancel action"),
        Line::from(""),
        Line::from(Span::styled(
            "Navigation",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from("↑/↓ or j/k: move action selection"),
        Line::from("enter: run selected action"),
        Line::from("mouse: hover + scroll; click headers to collapse"),
        Line::from("s: open settings panel"),
        Line::from(""),
        Line::from(Span::styled("Logs", Style::default().add_modifier(Modifier::BOLD))),
        Line::from(format!(
            "f: filter logs (current: {})",
            app.ui.log_config.filter.as_str()
        )),
        Line::from("hover Log Controls for menu"),
        Line::from("w: toggle watch refresh"),
        Line::from("p (hold 3s): pause stream + pin tooltip"),
        Line::from("u (hold 3s): unpin tooltip"),
    ]
}
