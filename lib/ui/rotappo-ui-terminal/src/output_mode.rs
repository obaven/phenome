//! Output mode selection for CLI formatters.

/// Output modes supported by CLI formatting.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputMode {
    /// Human-readable, line-oriented output.
    Plain,
    /// Pretty-printed JSON output.
    Json,
    /// Newline-delimited JSON output.
    Ndjson,
}

impl OutputMode {
    /// Parse a CLI string into an output mode.
    ///
    /// # Examples
    /// ```rust
    /// use rotappo_ui_terminal::OutputMode;
    ///
    /// assert_eq!(OutputMode::from_str("json"), Some(OutputMode::Json));
    /// assert_eq!(OutputMode::from_str("nope"), None);
    /// ```
    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "plain" => Some(OutputMode::Plain),
            "json" => Some(OutputMode::Json),
            "ndjson" => Some(OutputMode::Ndjson),
            _ => None,
        }
    }

    /// Return the canonical CLI string for this output mode.
    ///
    /// # Examples
    /// ```rust
    /// use rotappo_ui_terminal::OutputMode;
    ///
    /// assert_eq!(OutputMode::Ndjson.as_str(), "ndjson");
    /// ```
    pub fn as_str(self) -> &'static str {
        match self {
            OutputMode::Plain => "plain",
            OutputMode::Json => "json",
            OutputMode::Ndjson => "ndjson",
        }
    }
}
