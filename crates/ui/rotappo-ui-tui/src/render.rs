//! Frame rendering for the TUI.

use ratatui::layout::Rect;
use ratatui::prelude::Frame;
use ratatui::widgets::Clear;

use crate::app::App;
use crate::layout::{
    footer_spec, left_column_spec, middle_column_spec, action_header_spec, right_columns_spec,
    right_left_spec, right_right_spec, tui_shell_spec_with_footer, GridResolver, SLOT_ACTIONS,
    SLOT_AUX, SLOT_BODY, SLOT_CAPABILITIES, SLOT_FOOTER, SLOT_FOOTER_HELP, SLOT_FOOTER_SETTINGS,
    SLOT_HEADER, SLOT_LEFT, SLOT_LOGS, SLOT_LOG_CONTROLS, SLOT_MIDDLE, SLOT_ASSEMBLY,
    SLOT_ASSEMBLY_PROGRESS, SLOT_ASSEMBLY_STEPS, SLOT_PROBLEMS, SLOT_RIGHT, SLOT_RIGHT_LEFT,
    SLOT_RIGHT_RIGHT, SLOT_SNAPSHOT,
};

use super::panels;

pub(crate) fn render(frame: &mut Frame, app: &mut App) {
    let size = frame.area();
    app.ui.screen_area = size;
    let help_height = if app.panel_collapsed(crate::app::PanelId::Help) {
        2
    } else {
        size.height.saturating_mul(30).saturating_div(100).max(6)
    };
    let shell = GridResolver::resolve(size, &tui_shell_spec_with_footer(help_height));
    let header_area = shell
        .rect(SLOT_HEADER)
        .unwrap_or_else(|| Rect::new(0, 0, size.width, 3));
    let body_area = shell.rect(SLOT_BODY).unwrap_or_else(|| {
        Rect::new(0, 3, size.width, size.height.saturating_sub(help_height + 3))
    });
    let left_area = shell.rect(SLOT_LEFT).unwrap_or_else(|| {
        Rect::new(body_area.x, body_area.y, body_area.width / 3, body_area.height)
    });
    let middle_area = shell.rect(SLOT_MIDDLE).unwrap_or_else(|| {
        Rect::new(
            body_area.x + body_area.width / 3,
            body_area.y,
            body_area.width / 3,
            body_area.height,
        )
    });
    let right_area = shell.rect(SLOT_RIGHT).unwrap_or_else(|| {
        Rect::new(
            body_area.x + (body_area.width / 3) * 2,
            body_area.y,
            body_area.width / 3,
            body_area.height,
        )
    });
    let footer_area = shell.rect(SLOT_FOOTER).unwrap_or_else(|| {
        Rect::new(0, size.height.saturating_sub(help_height), size.width, help_height)
    });
    app.ui.body_area = body_area;

    panels::render_header(frame, header_area, app);

    let collapsed = 2u16;
    let action_progress_collapsed = app.panel_collapsed(crate::app::PanelId::AssemblyProgress);
    let snapshot_collapsed = app.panel_collapsed(crate::app::PanelId::Snapshot);
    let capabilities_collapsed = app.panel_collapsed(crate::app::PanelId::Capabilities);
    let assembly_steps_collapsed = app.panel_collapsed(crate::app::PanelId::AssemblySteps);
    let actions_collapsed = app.panel_collapsed(crate::app::PanelId::Actions);
    let problems_collapsed = app.panel_collapsed(crate::app::PanelId::Problems);
    let log_controls_collapsed = app.panel_collapsed(crate::app::PanelId::LogControls);
    let logs_collapsed = app.panel_collapsed(crate::app::PanelId::Logs);
    let help_open = !app.panel_collapsed(crate::app::PanelId::Help);
    let settings_open = !app.panel_collapsed(crate::app::PanelId::Settings);

    let left_spec = left_column_spec(
        action_progress_collapsed,
        snapshot_collapsed,
        capabilities_collapsed,
        collapsed,
    );
    let left_layout = GridResolver::resolve(
        left_area,
        &app.layout_policy.apply(&left_spec, left_area),
    );
    let left_action_area = left_layout.rect(SLOT_ASSEMBLY).unwrap_or(left_area);
    let left_cap_area = left_layout
        .rect(SLOT_CAPABILITIES)
        .unwrap_or_else(|| Rect::default());

    let action_header_spec =
        action_header_spec(action_progress_collapsed, snapshot_collapsed, collapsed);
    let action_header_layout = GridResolver::resolve(
        left_action_area,
        &app.layout_policy.apply(&action_header_spec, left_action_area),
    );
    let action_progress_area = action_header_layout
        .rect(SLOT_ASSEMBLY_PROGRESS)
        .unwrap_or(left_action_area);
    let snapshot_area = action_header_layout
        .rect(SLOT_SNAPSHOT)
        .unwrap_or_else(|| Rect::default());

    panels::render_assembly(frame, action_progress_area, snapshot_area, app);
    panels::render_capabilities(frame, left_cap_area, app);
    let middle_panel = app.middle_aux_panel();
    let middle_spec = middle_column_spec(assembly_steps_collapsed, collapsed);
    let middle_layout = GridResolver::resolve(
        middle_area,
        &app.layout_policy.apply(&middle_spec, middle_area),
    );
    let middle_action_area = middle_layout.rect(SLOT_ASSEMBLY_STEPS).unwrap_or(middle_area);
    if assembly_steps_collapsed {
        let middle_aux_area = middle_layout.rect(SLOT_AUX).unwrap_or(middle_area);
        match middle_panel {
            Some(crate::app::PanelId::Logs) => panels::render_logs(frame, middle_aux_area, app),
            Some(crate::app::PanelId::Help) => {
                panels::render_footer(frame, middle_aux_area, app)
            }
            _ => frame.render_widget(Clear, middle_aux_area),
        }
        panels::render_assembly_steps(frame, middle_action_area, app);
    } else {
        panels::render_assembly_steps(frame, middle_action_area, app);
    }

    let right_spec = right_columns_spec();
    let right_layout = GridResolver::resolve(right_area, &right_spec);
    let right_left_area = right_layout.rect(SLOT_RIGHT_LEFT).unwrap_or(right_area);
    let right_right_area = right_layout.rect(SLOT_RIGHT_RIGHT).unwrap_or(right_area);
    let right_left_spec = right_left_spec(actions_collapsed, problems_collapsed, collapsed);
    let right_left_layout = GridResolver::resolve(
        right_left_area,
        &app.layout_policy.apply(&right_left_spec, right_left_area),
    );
    let actions_area = right_left_layout.rect(SLOT_ACTIONS).unwrap_or(right_left_area);
    let problems_area = right_left_layout
        .rect(SLOT_PROBLEMS)
        .unwrap_or_else(|| Rect::default());
    let right_right_spec = right_right_spec(
        app.log_controls_height(),
        log_controls_collapsed,
        logs_collapsed,
        collapsed,
    );
    let right_right_layout = GridResolver::resolve(
        right_right_area,
        &app.layout_policy.apply(&right_right_spec, right_right_area),
    );
    let log_controls_area = right_right_layout
        .rect(SLOT_LOG_CONTROLS)
        .unwrap_or(right_right_area);
    let logs_area = right_right_layout
        .rect(SLOT_LOGS)
        .unwrap_or_else(|| Rect::default());

    panels::render_actions(frame, actions_area, app);
    panels::render_problems(frame, problems_area, app);
    panels::render_log_controls(frame, log_controls_area, app);
    if middle_panel != Some(crate::app::PanelId::Logs) {
        panels::render_logs(frame, logs_area, app);
    }
    app.ui.help_area = ratatui::layout::Rect::default();
    app.ui.settings_area = ratatui::layout::Rect::default();
    app.ui.settings_controls_row = None;
    let mut render_help = middle_panel != Some(crate::app::PanelId::Help);
    let mut render_settings = true;
    if help_open && !settings_open {
        render_settings = false;
    }
    if settings_open && !help_open {
        render_help = false;
    }
    let footer_spec = footer_spec();
    let footer_policy = app.layout_policy.clone();
    if let Some(slot) = crate::app::PanelId::Help.slot_id() {
        let slot_id = slot.into();
        footer_policy.clear_override(&slot_id);
    }
    if let Some(slot) = crate::app::PanelId::Settings.slot_id() {
        let slot_id = slot.into();
        footer_policy.clear_override(&slot_id);
    }
    if let Some(slot) = crate::app::PanelId::Help.slot_id() {
        footer_policy.set_visibility(slot, render_help);
    }
    if let Some(slot) = crate::app::PanelId::Settings.slot_id() {
        footer_policy.set_visibility(slot, render_settings);
    }
    if render_help && !render_settings {
        if let Some(slot) = crate::app::PanelId::Help.slot_id() {
            footer_policy.set_position(slot, 0, 0);
            footer_policy.set_span(slot, 1, 2);
        }
    }
    if render_settings && !render_help {
        if let Some(slot) = crate::app::PanelId::Settings.slot_id() {
            footer_policy.set_position(slot, 0, 0);
            footer_policy.set_span(slot, 1, 2);
        }
    }
    if render_help && render_settings {
        if help_open && !settings_open {
            if let Some(slot) = crate::app::PanelId::Help.slot_id() {
                footer_policy.set_position(slot, 0, 0);
                footer_policy.set_span(slot, 1, 2);
            }
        }
        if settings_open && !help_open {
            if let Some(slot) = crate::app::PanelId::Settings.slot_id() {
                footer_policy.set_position(slot, 0, 0);
                footer_policy.set_span(slot, 1, 2);
            }
        }
    }
    let footer_layout =
        GridResolver::resolve(footer_area, &footer_policy.apply(&footer_spec, footer_area));
    let footer_help_area = footer_layout.rect(SLOT_FOOTER_HELP).unwrap_or(footer_area);
    let footer_settings_area = footer_layout
        .rect(SLOT_FOOTER_SETTINGS)
        .unwrap_or(footer_area);
    if render_help {
        panels::render_footer(frame, footer_help_area, app);
    }
    if render_settings {
        panels::render_settings(frame, footer_settings_area, app);
    }

    panels::render_confirmation(frame, app);
    panels::render_tooltip(frame, app);
}
