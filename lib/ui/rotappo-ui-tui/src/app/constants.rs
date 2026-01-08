//! Internal constants for TUI app layout and log controls.

use rotappo_ui_presentation::logging::LOG_INTERVALS_SECS;

pub(crate) const COLLAPSED_HEIGHT: u16 = 2;
pub(crate) const LOG_CONTROLS_BASE_HEIGHT: u16 = 3;
pub(crate) const LOG_MENU_FILTER_LEN: u16 = 4;
pub(crate) const LOG_MENU_STREAM_LEN: u16 = (LOG_INTERVALS_SECS.len() + 1) as u16;
pub(crate) const FILTER_LABEL: &str = "Filter ";
pub(crate) const STREAM_LABEL: &str = "Stream ";
