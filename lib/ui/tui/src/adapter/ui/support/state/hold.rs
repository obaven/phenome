//! Hold-to-trigger input state.

use std::time::Instant;

/// Tracks the state of a key being held down.
#[derive(Debug, Clone)]
pub struct HoldState {
    pub key: char,
    pub started_at: Instant,
    pub triggered: bool,
}
