//! CLI output formatting helpers used by binaries.
//!
//! This module is CLI-only; UI adapters should prefer `presentation`
//! view-model helpers and must not depend on CLI output modes.

/// Output mode selection for CLI formatting.
mod output_mode;
/// Formatting helpers for CLI output modes.
mod format;

#[doc(inline)]
pub use format::{format_actions, format_assembly, format_events, format_problems, format_snapshot};
#[doc(inline)]
pub use output_mode::OutputMode;

#[cfg(any(feature = "bootstrappo-cli", feature = "rotato-cli"))]
pub mod cli;
