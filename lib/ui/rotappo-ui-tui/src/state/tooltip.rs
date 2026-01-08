//! Tooltip payloads used by overlays.

/// Tooltip content for hover or pinned overlays.
///
/// # Examples
/// ```rust
/// use rotappo_ui_tui::state::Tooltip;
///
/// let tip = Tooltip { title: "Demo".to_string(), lines: vec!["One".to_string()] };
/// assert_eq!(tip.lines.len(), 1);
/// ```
#[derive(Debug, Clone)]
pub struct Tooltip {
    pub title: String,
    pub lines: Vec<String>,
}
