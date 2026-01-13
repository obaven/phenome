//! Panel rendering entry points.

mod actions;
pub mod analytics;
mod assembly;
mod assembly_steps;
mod main;
pub mod bootstrap;
mod capabilities;
mod header;
mod help;
mod logs;
mod navbar;
mod notifications;
mod overlays;
mod problems;
mod settings;

pub use actions::render_actions;
pub use assembly::render_assembly;
pub use assembly_steps::render_assembly_steps;
pub use main::render_main;
pub use capabilities::render_capabilities;
pub use header::render_header;
pub use help::render_footer;
pub use logs::{render_log_controls, render_logs};
pub use navbar::render_navbar;
pub use notifications::render_notifications;
pub use overlays::{render_confirmation, render_tooltip};
pub use problems::render_problems;
pub use settings::render_settings;
