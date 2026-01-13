//! Shell grid specifications for the TUI layout.

use crate::layout::{GridSpec, TrackSize};

use super::slots::*;

const NAVBAR_WIDTH: u16 = 10;

/// Build the default shell grid spec.
///
/// # Examples
/// ```rust
/// use rotappo_ui_tui::layout::tui_shell_spec;
///
/// let spec = tui_shell_spec();
/// assert_eq!(spec.rows.len(), 2);
/// ```
pub fn tui_shell_spec() -> GridSpec {
    tui_shell_spec_with_footer(4)
}

/// Build the shell grid spec with a custom footer height.
pub fn tui_shell_spec_with_footer(footer_height: u16) -> GridSpec {
    let slots = crate::grid_slots!(
        crate::grid_slot!(SLOT_BODY, 0, 0, min: (24, 8)),
        crate::grid_slot!(SLOT_FOOTER, 1, 0, min: (24, 4)),
        crate::grid_slot!(SLOT_NAVBAR, 0, 1, span: (2, 1), min: (6, 8)),
    );
    crate::grid_spec!(
        rows: [TrackSize::Fill(1), TrackSize::Fixed(footer_height.max(2))],
        cols: [TrackSize::Fill(1), TrackSize::Fixed(NAVBAR_WIDTH)],
        slots: slots
    )
}

/// Build the left column grid based on collapsed state.
pub fn left_column_spec(
    action_progress_collapsed: bool,
    snapshot_collapsed: bool,
    collapsed_capabilities: bool,
    collapsed_height: u16,
) -> GridSpec {
    let action_progress_height = if action_progress_collapsed {
        collapsed_height
    } else {
        3
    };
    let snapshot_height = if snapshot_collapsed {
        collapsed_height
    } else {
        4
    };
    let left_top_height = action_progress_height.saturating_add(snapshot_height);
    let top = if collapsed_capabilities {
        TrackSize::Min(left_top_height)
    } else {
        TrackSize::Fixed(left_top_height)
    };
    let bottom = if collapsed_capabilities {
        TrackSize::Fixed(collapsed_height)
    } else {
        TrackSize::Fill(1)
    };
    crate::grid_spec!(
        rows: [top, bottom],
        cols: [TrackSize::Fill(1)],
        slots: [
            crate::grid_slot!(SLOT_ASSEMBLY, 0, 0, min: (12, 4)),
            crate::grid_slot!(SLOT_CAPABILITIES, 1, 0, min: (12, 4)),
        ]
    )
}

/// Build the action header grid based on collapsed state.
pub fn action_header_spec(
    action_progress_collapsed: bool,
    snapshot_collapsed: bool,
    collapsed_height: u16,
) -> GridSpec {
    let action_progress_track = if action_progress_collapsed {
        TrackSize::Fixed(collapsed_height)
    } else if snapshot_collapsed {
        TrackSize::Min(3)
    } else {
        TrackSize::Fixed(3)
    };
    let snapshot_track = if snapshot_collapsed {
        TrackSize::Fixed(collapsed_height)
    } else {
        TrackSize::Min(4)
    };
    crate::grid_spec!(
        rows: [action_progress_track, snapshot_track],
        cols: [TrackSize::Fill(1)],
        slots: [
            crate::grid_slot!(SLOT_ASSEMBLY_PROGRESS, 0, 0, min: (12, 3)),
            crate::grid_slot!(SLOT_SNAPSHOT, 1, 0, min: (12, 4)),
        ]
    )
}

/// Build the middle column grid based on collapsed state.
pub fn middle_column_spec(assembly_steps_collapsed: bool, collapsed_height: u16) -> GridSpec {
    if assembly_steps_collapsed {
        crate::grid_spec!(
            rows: [TrackSize::Fill(1), TrackSize::Fixed(collapsed_height)],
            cols: [TrackSize::Fill(1)],
            slots: [
                crate::grid_slot!(SLOT_AUX, 0, 0, min: (16, 4)),
                crate::grid_slot!(SLOT_ASSEMBLY_STEPS, 1, 0, min: (16, 2)),
            ]
        )
    } else {
        crate::grid_spec!(
            rows: [TrackSize::Fill(1)],
            cols: [TrackSize::Fill(1)],
            slots: [crate::grid_slot!(SLOT_ASSEMBLY_STEPS, 0, 0, min: (16, 8))]
        )
    }
}

/// Build the right column split grid.
pub fn right_columns_spec() -> GridSpec {
    crate::grid_spec!(
        rows: [TrackSize::Fill(1)],
        cols: [TrackSize::Percent(45), TrackSize::Percent(55)],
        slots: [
            crate::grid_slot!(SLOT_RIGHT_LEFT, 0, 0, min: (12, 8)),
            crate::grid_slot!(SLOT_RIGHT_RIGHT, 0, 1, min: (12, 8)),
        ]
    )
}

/// Build the right-left grid based on collapsed state.
pub fn right_left_spec(
    collapsed_actions: bool,
    collapsed_problems: bool,
    collapsed_height: u16,
) -> GridSpec {
    let (actions_track, problems_track) = match (collapsed_actions, collapsed_problems) {
        (true, true) => (
            TrackSize::Fixed(collapsed_height),
            TrackSize::Fixed(collapsed_height),
        ),
        (true, false) => (TrackSize::Fixed(collapsed_height), TrackSize::Fill(1)),
        (false, true) => (TrackSize::Fill(1), TrackSize::Fixed(collapsed_height)),
        (false, false) => (TrackSize::Min(8), TrackSize::Min(4)),
    };
    crate::grid_spec!(
        rows: [actions_track, problems_track],
        cols: [TrackSize::Fill(1)],
        slots: [
            crate::grid_slot!(SLOT_ACTIONS, 0, 0, min: (12, 4)),
            crate::grid_slot!(SLOT_PROBLEMS, 1, 0, min: (12, 4)),
        ]
    )
}

/// Build the right-right grid based on collapsed state.
pub fn right_right_spec(
    log_controls_height: u16,
    collapsed_log_controls: bool,
    collapsed_logs: bool,
    collapsed_height: u16,
) -> GridSpec {
    let (controls_track, logs_track) = match (collapsed_log_controls, collapsed_logs) {
        (true, true) => (
            TrackSize::Fixed(collapsed_height),
            TrackSize::Fixed(collapsed_height),
        ),
        (true, false) => (TrackSize::Fixed(collapsed_height), TrackSize::Fill(1)),
        (false, true) => (TrackSize::Fill(1), TrackSize::Fixed(collapsed_height)),
        (false, false) => (TrackSize::Fixed(log_controls_height), TrackSize::Min(6)),
    };
    crate::grid_spec!(
        rows: [controls_track, logs_track],
        cols: [TrackSize::Fill(1)],
        slots: [
            crate::grid_slot!(SLOT_LOG_CONTROLS, 0, 0, min: (12, 3)),
            crate::grid_slot!(SLOT_LOGS, 1, 0, min: (12, 6)),
        ]
    )
}

/// Build the footer grid.
pub fn footer_spec() -> GridSpec {
    crate::grid_spec!(
        rows: [TrackSize::Fill(1)],
        cols: [TrackSize::Percent(35), TrackSize::Percent(65)],
        slots: [
            crate::grid_slot!(SLOT_FOOTER_HELP, 0, 0, min: (12, 2)),
            crate::grid_slot!(SLOT_FOOTER_SETTINGS, 0, 1, min: (12, 2)),
        ]
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::GridResolver;
    use ratatui::layout::Rect;

    #[test]
    fn resolves_shell_spec() {
        let spec = tui_shell_spec();
        let layout = GridResolver::resolve(Rect::new(0, 0, 120, 40), &spec);
        let footer = layout.rect(SLOT_FOOTER).expect("footer");
        let body = layout.rect(SLOT_BODY).expect("body");
        let navbar = layout.rect(SLOT_NAVBAR).expect("navbar");

        assert_eq!(footer.height, 4);
        assert_eq!(body.height, 36);
        assert_eq!(navbar.height, 40);
        assert_eq!(body.x, 0);
        assert_eq!(navbar.x, body.width);
    }
}
