//! TUI application state and event handling.
//!
//! # Examples
//! ```rust,no_run
//! use rotappo_application::Runtime;
//! use rotappo_domain::ActionRegistry;
//! use rotappo_ui_tui::app::App;
//! use rotappo_ui_tui::app::AppContext;
//! use rotappo_ports::PortSet;
//!
//! let runtime = Runtime::new_with_ports(ActionRegistry::default(), PortSet::empty());
//! let context = AppContext::new("localhost", "config.yml", "assembly.yml", PortSet::empty());
//! let mut app = App::new(runtime, context);
//! app.on_tick();
//! ```

mod actions;
mod collapse;
mod constants;
mod core;
mod hover;
mod input;
mod keyboard;
mod layout;
mod lifecycle;
mod panel;
mod scroll;
mod tooltips;

#[doc(inline)]
pub use panel::PanelId;

#[doc(inline)]
pub use core::{App, AppContext, ConfirmPrompt};
pub(crate) use constants::{
    COLLAPSED_HEIGHT, FILTER_LABEL, LOG_CONTROLS_BASE_HEIGHT, LOG_MENU_FILTER_LEN,
    LOG_MENU_STREAM_LEN, STREAM_LABEL,
};
