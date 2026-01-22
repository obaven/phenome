//! Confirmation overlay rendering.

use ratatui::{
    prelude::Frame,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
};

use crate::app::App;
use crate::util::centered_rect;

/// Render the confirmation overlay if needed.
///
/// # Examples
/// ```rust,no_run
/// use ratatui::{backend::TestBackend, Terminal};
/// use phenome_ui_tui::app::App;
/// use phenome_ui_tui::panels::render_confirmation;
///
/// # fn app() -> App { todo!() }
/// let backend = TestBackend::new(80, 24);
/// let mut terminal = Terminal::new(backend).unwrap();
/// let mut app = app();
/// terminal.draw(|frame| render_confirmation(frame, &mut app)).unwrap();
/// ```
pub fn render_confirmation(frame: &mut Frame, app: &mut App) {
    let Some(confirm) = &app.confirm else {
        return;
    };

    let area = centered_rect(60, 30, frame.area());
    frame.render_widget(Clear, area);

    let lines = vec![
        Line::from(Span::styled(
            "Confirm Action",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(format!("Action: {label}", label = confirm.label.as_str())),
        Line::from(format!(
            "Safety: {safety}",
            safety = confirm.safety.as_str()
        )),
        Line::from(""),
        Line::from("Press Y to confirm, N to cancel"),
    ];

    let panel = Paragraph::new(lines)
        .block(Block::default().title("Confirmation").borders(Borders::ALL))
        .alignment(ratatui::prelude::Alignment::Center)
        .wrap(Wrap { trim: true });
    frame.render_widget(panel, area);
}
