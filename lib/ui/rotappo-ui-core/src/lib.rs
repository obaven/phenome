//! Framework-agnostic UI contracts.
//!
//! This module defines UI core types that can be reused by any
//! interface adapter (TUI, web, desktop) without pulling in ratatui
//! or terminal-specific dependencies (`rotappo-ui-terminal`).

pub mod actions;
pub mod geometry;
pub mod input;
pub mod panel;
pub mod state;

pub use actions::UiIntent;
pub use geometry::{UiMargin, UiPoint, UiRect};
pub use input::{
    UiInputEvent, UiKeyCode, UiKeyEvent, UiKeyModifiers, UiKeyState, UiMouseButton, UiMouseEvent,
    UiMouseKind,
};
pub use panel::UiPanelId;
pub use state::{
    UiHoldState, UiHoverPanel, UiLayoutState, UiLogMenuMode, UiTooltip, UiViewState,
};
