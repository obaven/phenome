//! Layout primitives and policy helpers for the TUI.

pub mod dsl;
pub mod grid;
pub mod policy;
pub mod renderer;
pub mod shell_spec;

pub use grid::{
    GridCache, GridLayout, GridResolver, GridSlot, GridSpec, SlotId, SpinGuard, SpinLock, TrackSize,
};
pub use policy::{GroupPolicy, LayoutPolicy, PanelPriority, SlotOverride, SlotPolicy};
pub use renderer::LayoutRenderer;
pub use shell_spec::{
    SLOT_ACTIONS, SLOT_ASSEMBLY, SLOT_ASSEMBLY_PROGRESS, SLOT_ASSEMBLY_STEPS, SLOT_AUX, SLOT_BODY,
    SLOT_CAPABILITIES, SLOT_FOOTER, SLOT_FOOTER_HELP, SLOT_FOOTER_SETTINGS, SLOT_HEADER, SLOT_LEFT,
    SLOT_LOG_CONTROLS, SLOT_LOGS, SLOT_MIDDLE, SLOT_NAVBAR, SLOT_NOTIFICATIONS, SLOT_PROBLEMS,
    SLOT_RIGHT, SLOT_RIGHT_LEFT, SLOT_RIGHT_RIGHT, SLOT_SNAPSHOT, action_header_spec, footer_spec,
    left_column_spec, middle_column_spec, right_columns_spec, right_left_spec, right_right_spec,
    tui_shell_spec, tui_shell_spec_with_footer,
};
