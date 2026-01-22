//! TUI application state and event handling.
//!
//! # Examples
//! ```rust,no_run
//! use phenome_application::Runtime;
//! use phenome_domain::ActionRegistry;
//! use phenome_ui_tui::app::App;
//! use phenome_ui_tui::app::AppContext;
//! use phenome_ports::PortSet;
//!
//! let runtime = Runtime::new_with_ports(ActionRegistry::default(), PortSet::empty());
//! let context = AppContext::new("localhost", "config.yml", "assembly.yml", PortSet::empty());
//! let mut app = App::new(runtime, context);
//! app.on_tick();
//! ```

pub mod behavior;
pub mod input;
pub mod layout;
pub mod state;

pub use behavior::{actions, core, graph, lifecycle};
pub use input::{input as process_input, keyboard};
pub use layout::{layout as update_layout, navigation, panel};
pub use state::{collapse, hover, scroll, tooltips};

pub(crate) use graph::{GraphDirection, GraphRenderState, TerminalImageProtocol};
#[doc(inline)]
pub use navigation::{NavAction, NavSection, NavSubItem, NavView, nav_items};
#[doc(inline)]
pub use panel::PanelId;

#[doc(inline)]
pub use core::{App, AppContext, ConfirmPrompt};
