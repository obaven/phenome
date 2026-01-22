//! Framework-agnostic UI contracts.
//!
//! This module defines UI core types that can be reused by any
//! interface adapter (TUI, web, desktop) without pulling in ratatui
//! or terminal-specific dependencies (`phenome-ui-terminal`).

mod core;

pub use core::{actions, geometry, input, panel, state};

pub use actions::UiIntent;
pub use geometry::{UiMargin, UiPoint, UiRect};
pub use input::{
    UiInputEvent, UiKeyCode, UiKeyEvent, UiKeyModifiers, UiKeyState, UiMouseButton, UiMouseEvent,
    UiMouseKind,
};
pub use panel::UiPanelId;
pub use state::{UiHoldState, UiHoverPanel, UiLayoutState, UiLogMenuMode, UiTooltip, UiViewState};
