use crate::bootstrap::state::BootstrapUiState;
use crate::util::{centered_rect, format_age};
use ratatui::layout::{Margin, Rect};
use ratatui::prelude::Frame;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{
    Block, Borders, Clear, List, ListItem, Scrollbar, ScrollbarOrientation, ScrollbarState,
};
use phenome_domain::EventLevel;

pub fn render(frame: &mut Frame, area: Rect, ui: &mut BootstrapUiState) {
    let overlay_area = centered_rect(80, 70, area);
    frame.render_widget(Clear, overlay_area);

    let inner_height = overlay_area.height.saturating_sub(2) as usize;
    ui.log_view_height = inner_height;

    let total = ui.log_events.len();
    let max_offset = total.saturating_sub(inner_height);
    let offset = ui.log_scroll.min(max_offset);
    let start = total.saturating_sub(inner_height).saturating_sub(offset);

    let items: Vec<ListItem> = ui
        .log_events
        .iter()
        .skip(start)
        .map(|event| {
            let level_style = match event.level {
                EventLevel::Info => Style::default().fg(Color::Cyan),
                EventLevel::Warn => Style::default().fg(Color::Yellow),
                EventLevel::Error => Style::default().fg(Color::Red),
            };
            let line = Line::from(vec![
                Span::styled(
                    format!("[{level}]", level = event.level.as_str()),
                    level_style,
                ),
                Span::raw(" "),
                Span::raw(format_age(event.timestamp_ms)),
                Span::raw(" "),
                Span::raw(&event.message),
            ]);
            ListItem::new(line)
        })
        .collect();

    let title = format!("Bootstrap Logs ({total} events, q to close)");
    let list = List::new(items).block(Block::default().title(title).borders(Borders::ALL));
    frame.render_widget(list, overlay_area);

    if total > inner_height && inner_height > 0 {
        let mut state = ScrollbarState::new(total).position(offset);
        let bar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .style(Style::default().fg(Color::Cyan));
        frame.render_stateful_widget(
            bar,
            overlay_area.inner(Margin {
                horizontal: 0,
                vertical: 1,
            }),
            &mut state,
        );
    }
}
