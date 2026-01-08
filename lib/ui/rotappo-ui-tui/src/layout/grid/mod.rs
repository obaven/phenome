//! Grid layout primitives for the TUI shell.
//!
//! # Examples
//! ```rust
//! use ratatui::layout::Rect;
//! use rotappo_ui_tui::layout::{GridResolver, GridSpec, GridSlot, TrackSize};
//!
//! let spec = GridSpec::new(
//!     vec![TrackSize::Percent(50), TrackSize::Percent(50)],
//!     vec![TrackSize::Percent(100)],
//! )
//! .with_slots(vec![GridSlot::new("header", 0, 0), GridSlot::new("body", 1, 0)]);
//! let layout = GridResolver::resolve(Rect::new(0, 0, 100, 40), &spec);
//! assert!(layout.rect("header").is_some());
//! ```

mod cache;
mod layout;
mod resolver;
mod slot;
mod spec;
mod spin_lock;
mod track;

pub use cache::GridCache;
pub use layout::GridLayout;
pub use resolver::GridResolver;
pub use slot::{GridSlot, SlotId};
pub use spec::GridSpec;
pub use spin_lock::{SpinGuard, SpinLock};
pub use track::TrackSize;

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::layout::Rect;

    #[test]
    fn resolves_basic_grid() {
        let spec = GridSpec::new(
            vec![TrackSize::Percent(50), TrackSize::Percent(50)],
            vec![TrackSize::Percent(50), TrackSize::Percent(50)],
        )
        .with_slots(vec![
            GridSlot::new("a", 0, 0),
            GridSlot::new("b", 0, 1),
            GridSlot::new("c", 1, 0).span(1, 2),
        ]);

        let area = Rect::new(0, 0, 100, 40);
        let layout = GridResolver::resolve(area, &spec);
        let a = layout.rect("a").expect("slot a");
        let b = layout.rect("b").expect("slot b");
        let c = layout.rect("c").expect("slot c");

        assert_eq!(a.width, 50);
        assert_eq!(b.x, 50);
        assert_eq!(c.width, 100);
        assert_eq!(c.y, 20);
    }

    #[test]
    fn skips_hidden_slots() {
        let spec = GridSpec::new(
            vec![TrackSize::Fixed(2)],
            vec![TrackSize::Fixed(2)],
        )
        .with_slots(vec![GridSlot::new("hidden", 0, 0).hidden()]);
        let layout = GridResolver::resolve(Rect::new(0, 0, 4, 4), &spec);
        assert!(layout.rect("hidden").is_none());
    }

    #[test]
    fn applies_min_max_and_offsets() {
        let spec = GridSpec::new(vec![TrackSize::Fixed(10)], vec![TrackSize::Fixed(10)])
            .with_slots(vec![GridSlot::new("slot", 0, 0)
                .movable(true)
                .with_min_size(6, 6)
                .with_max_size(8, 8)
                .offset(3, 4)]);
        let area = Rect::new(0, 0, 10, 10);
        let layout = GridResolver::resolve(area, &spec);
        let rect = layout.rect("slot").expect("slot rect");
        assert_eq!(rect.width, 8);
        assert_eq!(rect.height, 8);
        assert_eq!(rect.x, 2);
        assert_eq!(rect.y, 2);
    }

    #[test]
    fn clamps_offsets_to_area() {
        let spec = GridSpec::new(vec![TrackSize::Fixed(6)], vec![TrackSize::Fixed(6)])
            .with_slots(vec![GridSlot::new("slot", 0, 0)
                .movable(true)
                .with_max_size(4, 4)
                .offset(10, 10)]);
        let area = Rect::new(0, 0, 6, 6);
        let layout = GridResolver::resolve(area, &spec);
        let rect = layout.rect("slot").expect("slot rect");
        assert_eq!(rect.width, 4);
        assert_eq!(rect.height, 4);
        assert_eq!(rect.x, 2);
        assert_eq!(rect.y, 2);
    }
}
