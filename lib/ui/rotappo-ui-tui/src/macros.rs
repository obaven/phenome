//! UI macro helpers for panel styling and layout splits.
//!
//! # Examples
//! ```rust
//! use ratatui::layout::{Constraint, Direction, Rect};
//!
//! let area = Rect::new(0, 0, 10, 4);
//! let _rows = rotappo_ui_tui::ui_layout_split!(
//!     Direction::Vertical,
//!     [Constraint::Percentage(50), Constraint::Percentage(50)],
//!     area
//! );
//! ```

#[macro_export]
macro_rules! ui_panel_block {
    ($title:expr, $hovered:expr) => {{
        let mut block = ratatui::widgets::Block::default()
            .title($title)
            .borders(ratatui::widgets::Borders::ALL);
        if $hovered {
            block = block.style(
                ratatui::style::Style::default().bg(ratatui::style::Color::Rgb(0, 90, 90)),
            );
        }
        block
    }};
    ($title:expr, $hovered:expr, $pulse:expr) => {{
        let mut block = ratatui::widgets::Block::default()
            .title($title)
            .borders(ratatui::widgets::Borders::ALL);
        if $pulse {
            block = block.style(ratatui::style::Style::default().fg(ratatui::style::Color::Cyan));
        }
        if $hovered {
            block = block.style(
                ratatui::style::Style::default().bg(ratatui::style::Color::Rgb(0, 90, 90)),
            );
        }
        block
    }};
}

#[macro_export]
macro_rules! ui_layout_split {
    ($direction:expr, [$($constraint:expr),+ $(,)?], $area:expr) => {{
        ratatui::layout::Layout::default()
            .direction($direction)
            .constraints([$($constraint),+])
            .split($area)
    }};
}
