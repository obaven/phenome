//! CLI output formatting helpers used by binaries.
//!
//! This module is CLI-only; UI adapters should prefer `presentation`
//! view-model helpers and must not depend on CLI output modes.

/// Formatting helpers for CLI output modes.
mod format;
/// Output mode selection for CLI formatting.
mod output_mode;

#[doc(inline)]
pub use format::{
    format_actions, format_assembly, format_events, format_problems, format_snapshot,
};
#[doc(inline)]
pub use output_mode::OutputMode;

#[cfg(any(feature = "primer-cli", feature = "plasmid-cli"))]
pub mod cli;
