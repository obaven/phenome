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
    fn parse(value: &str) -> Option<Self> {
        match value {
            "plain" => Some(OutputMode::Plain),
            "json" => Some(OutputMode::Json),
            "ndjson" => Some(OutputMode::Ndjson),
            _ => None,
        }
    }

    /// Parse a CLI string into an output mode.
    ///
    /// # Examples
    /// ```rust
    /// use phenome_ui_terminal::OutputMode;
    ///
    /// assert_eq!(OutputMode::parse_cli("json"), Some(OutputMode::Json));
    /// assert_eq!(OutputMode::parse_cli("nope"), None);
    /// ```
    pub fn parse_cli(value: &str) -> Option<Self> {
        Self::parse(value)
    }

    /// Return the canonical CLI string for this output mode.
    ///
    /// # Examples
    /// ```rust
    /// use phenome_ui_terminal::OutputMode;
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

impl std::str::FromStr for OutputMode {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::parse(value).ok_or_else(|| format!("Unknown output mode: {value}"))
    }
}
