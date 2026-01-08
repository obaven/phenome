//! UI utility helpers for layout, time formatting, and styling.
//!
//! # Examples
//! ```rust
//! use rotappo_ui_tui::util::format_age;
//!
//! let _ = format_age(0);
//! ```

mod color;
mod assembly;
mod problems;
mod rect;
mod time;
mod tooltip;

pub use color::{animated_color, traveling_glow};
pub use assembly::{capability_icon, assembly_lines, assembly_status_icon, AssemblyLine};
pub use problems::collect_problems;
pub use rect::{anchored_rect, anchored_rect_with_offset, centered_rect};
pub use time::{format_age, spinner_frame};
pub use tooltip::{tooltip_rect_for_mouse, tooltip_rect_in_corner};
