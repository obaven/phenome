//! Shell grid specifications for the TUI layout.

use crate::layout::{GridSpec, TrackSize};

use super::slots::*;

const NAVBAR_WIDTH: u16 = 10;

/// Build the default shell grid spec.
///
/// # Examples
/// ```rust
/// use phenome_ui_tui::layout::tui_shell_spec;
///
/// let spec = tui_shell_spec();
/// assert_eq!(spec.rows.len(), 2);
/// ```
pub fn tui_shell_spec() -> GridSpec {
    tui_shell_spec_with_footer(4)
}

/// Build the shell grid spec with a custom footer height.
pub fn tui_shell_spec_with_footer(footer_height: u16) -> GridSpec {
    let slots = crate::grid_slots!(
        crate::grid_slot!(SLOT_BODY, 0, 0, min: (24, 8)),
        crate::grid_slot!(SLOT_FOOTER, 1, 0, min: (24, 4)),
        crate::grid_slot!(SLOT_NAVBAR, 0, 1, span: (2, 1), min: (6, 8)),
    );
    crate::grid_spec!(
        rows: [TrackSize::Fill(1), TrackSize::Fixed(footer_height.max(2))],
        cols: [TrackSize::Fill(1), TrackSize::Fixed(NAVBAR_WIDTH)],
        slots: slots
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::GridResolver;
    use ratatui::layout::Rect;

    #[test]
    fn resolves_shell_spec() {
        let spec = tui_shell_spec();
        let layout = GridResolver::resolve(Rect::new(0, 0, 120, 40), &spec);
        let footer = layout.rect(SLOT_FOOTER).expect("footer");
        let body = layout.rect(SLOT_BODY).expect("body");
        let navbar = layout.rect(SLOT_NAVBAR).expect("navbar");

        assert_eq!(footer.height, 4);
        assert_eq!(body.height, 36);
        assert_eq!(navbar.height, 40);
        assert_eq!(body.x, 0);
        assert_eq!(navbar.x, body.width);
    }
}
