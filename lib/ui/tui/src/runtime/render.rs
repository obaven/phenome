//! Frame rendering for the TUI.

use ratatui::layout::Rect;
use ratatui::prelude::Frame;
use ratatui::widgets::Clear;

use crate::app::{App, PanelId};
use crate::layout::{
    GridResolver, SLOT_BODY, SLOT_FOOTER, SLOT_NAVBAR, tui_shell_spec_with_footer,
};

use crate::adapter::ui::panels;

pub(crate) fn render(frame: &mut Frame, app: &mut App) {
    let size = frame.area();
    app.ui.screen_area = size;
    let help_height = if app.panel_collapsed(PanelId::Help) {
        2
    } else {
        6
    };
    let shell = GridResolver::resolve(size, &tui_shell_spec_with_footer(help_height));
    let body_area = shell
        .rect(SLOT_BODY)
        .unwrap_or_else(|| Rect::new(0, 0, size.width, size.height.saturating_sub(help_height)));
    let footer_area = shell.rect(SLOT_FOOTER).unwrap_or_else(|| {
        Rect::new(
            0,
            size.height.saturating_sub(help_height),
            size.width,
            help_height,
        )
    });
    let navbar_area = shell.rect(SLOT_NAVBAR).unwrap_or_default();

    panels::render_main(frame, body_area, app);
    panels::render_footer(frame, footer_area, app);
    panels::render_navbar(frame, navbar_area, app);

    let notifications_open = !app.panel_collapsed(PanelId::Notifications);
    if notifications_open {
        let width = body_area.width.min(40).max(24).min(body_area.width);
        let height = body_area.height.min(20).max(8).min(body_area.height);
        let x = body_area
            .x
            .saturating_add(body_area.width.saturating_sub(width));
        let y = body_area
            .y
            .saturating_add(body_area.height.saturating_sub(height) / 2);
        let overlay_area = Rect::new(x, y, width, height);
        frame.render_widget(Clear, overlay_area);
        panels::render_notifications(frame, overlay_area);
    }

    panels::render_confirmation(frame, app);
    panels::render_tooltip(frame, app);
}
