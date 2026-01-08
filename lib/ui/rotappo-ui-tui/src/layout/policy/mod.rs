//! Layout policy rules for grid slots.
//!
//! # Examples
//! ```rust
//! use ratatui::layout::Rect;
//! use rotappo_ui_tui::layout::{GridSpec, GridSlot, LayoutPolicy, PanelPriority, SlotPolicy, TrackSize};
//!
//! let policy = LayoutPolicy::new();
//! policy.set_policy("slot", SlotPolicy::new(PanelPriority::High).min_size(4, 2));
//! let spec = GridSpec::new(vec![TrackSize::Fixed(4)], vec![TrackSize::Fixed(4)])
//!     .with_slots(vec![GridSlot::new("slot", 0, 0)]);
//! let updated = policy.apply(&spec, Rect::new(0, 0, 4, 4));
//! assert_eq!(updated.slots[0].min_width, Some(4));
//! ```

mod group;
mod layout;
mod priority;
mod slot;
mod state;

pub use group::GroupPolicy;
pub use layout::LayoutPolicy;
pub use priority::PanelPriority;
pub use slot::{SlotOverride, SlotPolicy};
