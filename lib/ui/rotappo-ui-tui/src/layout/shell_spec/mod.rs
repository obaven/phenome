//! Shell-level layout specifications and slot identifiers.

mod slots;
mod specs;

pub use slots::{
    SLOT_ACTIONS, SLOT_AUX, SLOT_BODY, SLOT_CAPABILITIES, SLOT_FOOTER, SLOT_FOOTER_HELP,
    SLOT_FOOTER_SETTINGS, SLOT_HEADER, SLOT_LEFT, SLOT_LOGS, SLOT_LOG_CONTROLS, SLOT_MIDDLE,
    SLOT_ASSEMBLY, SLOT_ASSEMBLY_PROGRESS, SLOT_ASSEMBLY_STEPS, SLOT_PROBLEMS, SLOT_RIGHT, SLOT_RIGHT_LEFT,
    SLOT_RIGHT_RIGHT, SLOT_SNAPSHOT,
};
pub use specs::{
    footer_spec, left_column_spec, middle_column_spec, action_header_spec, right_columns_spec,
    right_left_spec, right_right_spec, tui_shell_spec, tui_shell_spec_with_footer,
};
