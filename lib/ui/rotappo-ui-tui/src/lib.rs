//! TUI entry points and rendering pipeline.
//!
//! # Examples
//! ```rust,no_run
//! use rotappo_application::Runtime;
//! use rotappo_domain::ActionRegistry;
//! use rotappo_ui_tui as tui;
//! use rotappo_ui_tui::app::AppContext;
//! use rotappo_ports::PortSet;
//! # fn main() -> anyhow::Result<()> {
//! let runtime = Runtime::new_with_ports(ActionRegistry::default(), PortSet::empty());
//! let context = AppContext::new("localhost", "config.yml", "assembly.yml", PortSet::empty());
//! tui::start(runtime, context)?;
//! # Ok(())
//! # }
//! ```

pub mod app;
pub mod layout;
pub mod macros;
pub mod panels;
pub mod state;
pub mod util;

mod render;
mod runner;

pub use runner::start;
