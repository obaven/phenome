//! Layout primitives and shell grid helpers for the TUI.

pub mod grid;
pub mod shell_spec;

pub use grid::{
    GridLayout, GridResolver, GridSlot, GridSpec, SlotId, TrackSize,
};
pub use shell_spec::{
    SLOT_BODY, SLOT_FOOTER, SLOT_NAVBAR, tui_shell_spec, tui_shell_spec_with_footer,
};
