//! Input events for UI-core adapters.

use super::geometry::UiPoint;

/// High-level input events for UI adapters.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiInputEvent {
    Key(UiKeyEvent),
    Mouse(UiMouseEvent),
    Tick,
}

/// Keyboard event.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UiKeyEvent {
    pub code: UiKeyCode,
    pub modifiers: UiKeyModifiers,
    pub state: UiKeyState,
}

/// Key code representation independent of any backend.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiKeyCode {
    Char(char),
    Enter,
    Esc,
    Tab,
    BackTab,
    Backspace,
    Delete,
    Insert,
    Up,
    Down,
    Left,
    Right,
    Home,
    End,
    PageUp,
    PageDown,
    F(u8),
}

/// Keyboard modifier flags.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct UiKeyModifiers {
    pub shift: bool,
    pub control: bool,
    pub alt: bool,
    pub meta: bool,
}

/// Key press state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiKeyState {
    Press,
    Release,
    Repeat,
}

/// Mouse event data.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UiMouseEvent {
    pub kind: UiMouseKind,
    pub position: UiPoint,
}

/// Mouse buttons supported by UI adapters.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiMouseButton {
    Left,
    Right,
    Middle,
}

/// Mouse event kinds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiMouseKind {
    Down(UiMouseButton),
    Up(UiMouseButton),
    Drag(UiMouseButton),
    Moved,
    ScrollUp,
    ScrollDown,
    ScrollLeft,
    ScrollRight,
}
