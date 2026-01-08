//! Time-based UI helpers.

use rotappo_domain::now_millis;

/// Frame used to animate a spinner.
///
/// # Examples
/// ```rust
/// use rotappo_ui_tui::util::spinner_frame;
///
/// let frame = spinner_frame();
/// assert!("|/-\\".contains(frame));
/// ```
pub fn spinner_frame() -> char {
    const FRAMES: [char; 4] = ['|', '/', '-', '\\'];
    let index = (now_millis() / 250) % FRAMES.len() as u64;
    FRAMES[index as usize]
}

/// Format an age string from a millisecond timestamp.
///
/// # Examples
/// ```rust
/// use rotappo_ui_tui::util::format_age;
///
/// let formatted = format_age(0);
/// assert!(formatted.ends_with("ago"));
/// ```
pub fn format_age(timestamp_ms: u64) -> String {
    let now = now_millis();
    let delta_ms = now.saturating_sub(timestamp_ms);
    let seconds = delta_ms / 1_000;
    if seconds < 60 {
        return format!("{}s ago", seconds);
    }
    let minutes = seconds / 60;
    if minutes < 60 {
        return format!("{}m ago", minutes);
    }
    let hours = minutes / 60;
    format!("{}h ago", hours)
}
