//! TUI entry points and rendering pipeline.
//!
//! # Examples
//! ```rust,no_run
//! use phenome_application::Runtime;
//! use phenome_domain::ActionRegistry;
//! use phenome_ui_tui as tui;
//! use phenome_ui_tui::app::AppContext;
//! use phenome_ports::PortSet;
//! # fn main() -> anyhow::Result<()> {
//! let runtime = Runtime::new_with_ports(ActionRegistry::default(), PortSet::empty());
//! let context = AppContext::new("localhost", "config.yml", "assembly.yml", PortSet::empty());
//! tui::start(runtime, context)?;
//! # Ok(())
//! # }
//! ```

mod adapter;
mod runtime;

pub use adapter::bootstrap;
pub use adapter::clients::analytics as analytics_client;
pub use adapter::ui::app;
pub use adapter::ui::layout;
pub use adapter::ui::panels;
pub use adapter::ui::support::macros;
pub use adapter::ui::support::state;
pub use adapter::ui::support::util;
pub use runtime::render;
pub use runtime::runner;
pub use runtime::terminal;

pub use bootstrap::start_bootstrap;
pub use runner::start;
