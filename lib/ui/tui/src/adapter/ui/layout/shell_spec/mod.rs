//! Shell-level layout specifications and slot identifiers.

mod slots;
mod specs;

pub use slots::{SLOT_BODY, SLOT_FOOTER, SLOT_NAVBAR};
pub use specs::{
    tui_shell_spec, tui_shell_spec_with_footer,
};
