//! Panel rendering entry points.

pub mod chrome;
pub mod overlays;
pub mod views;

pub use chrome::help::render_footer;
pub use chrome::navbar::render_navbar;
pub use chrome::notifications::render_notifications;
pub use overlays::{render_confirmation, render_tooltip};
pub use views::main::render_main;

pub use views::analytics;
