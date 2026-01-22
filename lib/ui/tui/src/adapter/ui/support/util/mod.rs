//! UI utility helpers for layout, time formatting, and styling.
//!
//! # Examples
//! ```rust
//! use phenome_ui_tui::util::format_age;
//!
//! let _ = format_age(0);
//! ```

mod data;
mod format;
mod geometry;

pub use data::assembly::{AssemblyLine, assembly_lines, assembly_status_icon, capability_icon};
pub use data::problems::collect_problems;
pub use format::color::{animated_color, traveling_glow};
pub use format::time::{format_age, spinner_frame};
pub use geometry::rect::{anchored_rect, anchored_rect_with_offset, centered_rect};
pub use geometry::tooltip::{tooltip_rect_for_mouse, tooltip_rect_in_corner};
