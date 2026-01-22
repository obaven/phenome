//! Shared UI state for TUI rendering and input handling.
//!
//! # Examples
//! ```rust
//! use phenome_ui_tui::state::UiState;
//!
//! let state = UiState::new();
//! assert!(state.mouse_pos.is_none());
//! ```

mod hold;
mod hover;
mod tooltip;
mod ui_state;

pub use hold::HoldState;
pub use hover::HoverPanel;
pub use tooltip::Tooltip;
pub use ui_state::UiState;
