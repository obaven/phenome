//! UI-core state and overlay types.

use std::time::Instant;

use phenome_domain::Event;
use phenome_ui_presentation::logging::LogStreamConfig;

use super::geometry::{UiPoint, UiRect};

/// Panel currently under the cursor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiHoverPanel {
    None,
    Assembly,
    Capabilities,
    Actions,
    Logs,
    Problems,
    Help,
    Settings,
}

/// Log menu modes for filtering and stream controls.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiLogMenuMode {
    Filter,
    Stream,
}

/// Tooltip content for hover or pinned overlays.
#[derive(Debug, Clone)]
pub struct UiTooltip {
    pub title: String,
    pub lines: Vec<String>,
}

/// Tracks the state of a key being held down.
#[derive(Debug, Clone)]
pub struct UiHoldState {
    pub key: char,
    pub started_at: Instant,
    pub triggered: bool,
}

/// Layout rectangles for UI panels.
#[derive(Debug, Clone)]
pub struct UiLayoutState {
    pub screen_area: UiRect,
    pub actions_area: UiRect,
    pub settings_area: UiRect,
    pub settings_gear_area: UiRect,
    pub log_controls_area: UiRect,
    pub assembly_area: UiRect,
    pub assembly_progress_area: UiRect,
    pub snapshot_area: UiRect,
    pub body_area: UiRect,
    pub capabilities_area: UiRect,
    pub logs_area: UiRect,
    pub problems_area: UiRect,
    pub help_area: UiRect,
    pub log_menu_area: UiRect,
    pub log_filter_tag_area: UiRect,
    pub log_stream_tag_area: UiRect,
}

impl UiLayoutState {
    /// Construct a default layout state.
    pub fn new() -> Self {
        Self {
            screen_area: UiRect::default(),
            actions_area: UiRect::default(),
            settings_area: UiRect::default(),
            settings_gear_area: UiRect::default(),
            log_controls_area: UiRect::default(),
            assembly_area: UiRect::default(),
            assembly_progress_area: UiRect::default(),
            snapshot_area: UiRect::default(),
            body_area: UiRect::default(),
            capabilities_area: UiRect::default(),
            logs_area: UiRect::default(),
            problems_area: UiRect::default(),
            help_area: UiRect::default(),
            log_menu_area: UiRect::default(),
            log_filter_tag_area: UiRect::default(),
            log_stream_tag_area: UiRect::default(),
        }
    }
}

impl Default for UiLayoutState {
    fn default() -> Self {
        Self::new()
    }
}

/// Aggregated UI view state shared across adapters.
#[derive(Debug, Clone)]
pub struct UiViewState {
    pub hover_action_definition_index: Option<usize>,
    pub hover_capability_index: Option<usize>,
    pub hover_action_index: Option<usize>,
    pub hover_problem_index: Option<usize>,
    pub log_menu_hover_index: Option<usize>,
    pub hover_snapshot: bool,
    pub hover_panel: UiHoverPanel,
    pub mouse_pos: Option<UiPoint>,
    pub collapsed_assembly_progress: bool,
    pub collapsed_snapshot: bool,
    pub collapsed_capabilities: bool,
    pub collapsed_assembly_steps: bool,
    pub collapsed_actions: bool,
    pub collapsed_settings: bool,
    pub collapsed_log_controls: bool,
    pub collapsed_problems: bool,
    pub collapsed_logs: bool,
    pub collapsed_help: bool,
    pub open_counter: u64,
    pub help_opened_at: Option<u64>,
    pub logs_opened_at: Option<u64>,
    pub settings_selected: usize,
    pub settings_controls_row: Option<u16>,
    pub log_menu_len: usize,
    pub log_menu_mode: Option<UiLogMenuMode>,
    pub log_menu_pinned: bool,
    pub auto_refresh: bool,
    pub log_config: LogStreamConfig,
    pub log_paused: bool,
    pub log_scroll: u16,
    pub assembly_scroll: u16,
    pub capabilities_scroll: u16,
    pub actions_scroll: u16,
    pub problems_scroll: u16,
    pub log_cache: Vec<Event>,
    pub last_log_emit: Instant,
    pub hold_state: Option<UiHoldState>,
    pub pinned_tooltip: Option<UiTooltip>,
    pub action_flash_index: Option<usize>,
    pub action_flash_at: Option<Instant>,
}

impl UiViewState {
    /// Construct a default view state aligned with the current TUI defaults.
    pub fn new() -> Self {
        Self {
            hover_action_definition_index: None,
            hover_capability_index: None,
            hover_action_index: None,
            hover_problem_index: None,
            log_menu_hover_index: None,
            hover_snapshot: false,
            hover_panel: UiHoverPanel::None,
            mouse_pos: None,
            collapsed_assembly_progress: false,
            collapsed_snapshot: false,
            collapsed_capabilities: false,
            collapsed_assembly_steps: false,
            collapsed_actions: false,
            collapsed_settings: true,
            collapsed_log_controls: false,
            collapsed_problems: false,
            collapsed_logs: false,
            collapsed_help: false,
            open_counter: 2,
            help_opened_at: Some(1),
            logs_opened_at: Some(2),
            settings_selected: 0,
            settings_controls_row: None,
            log_menu_len: 0,
            log_menu_mode: None,
            log_menu_pinned: false,
            auto_refresh: true,
            log_config: LogStreamConfig::default(),
            log_paused: false,
            log_scroll: 0,
            assembly_scroll: 0,
            capabilities_scroll: 0,
            actions_scroll: 0,
            problems_scroll: 0,
            log_cache: Vec::new(),
            last_log_emit: Instant::now(),
            hold_state: None,
            pinned_tooltip: None,
            action_flash_index: None,
            action_flash_at: None,
        }
    }
}

impl Default for UiViewState {
    fn default() -> Self {
        Self::new()
    }
}
