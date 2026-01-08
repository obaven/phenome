//! Log menu selection mode.

/// Log menu modes for filtering and stream controls.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogMenuMode {
    Filter,
    Stream,
}
