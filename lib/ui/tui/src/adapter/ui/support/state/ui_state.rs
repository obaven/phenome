//! Main UI state storage.

use ratatui::layout::Rect;
use std::time::Instant;

use phenome_domain::Event;
use phenome_ui_presentation::logging::LogStreamConfig;

use super::{HoldState, HoverPanel, Tooltip};

/// Aggregated UI state shared across panels and input handlers.
pub struct UiState {
    pub screen_area: Rect,
    pub actions_area: Rect,
    pub navbar_item_areas: [Rect; 3],
    pub nav_flyout_area: Rect,
    pub nav_flyout_item_areas: [Rect; 12],
    pub nav_flyout_count: usize,
    pub assembly_area: Rect,
    pub body_area: Rect,
    pub capabilities_area: Rect,
    pub logs_area: Rect,
    pub hover_capability_index: Option<usize>,
    pub hover_action_index: Option<usize>,
    pub hover_panel: HoverPanel,
    pub mouse_pos: Option<(u16, u16)>,
    pub collapsed_capabilities: bool,
    pub collapsed_assembly_steps: bool,
    pub collapsed_actions: bool,
    pub collapsed_logs: bool,
    pub collapsed_help: bool,
    pub collapsed_notifications: bool,
    pub auto_refresh: bool,
    pub log_config: LogStreamConfig,
    pub log_paused: bool,
    pub log_scroll: u16,
    pub assembly_scroll: u16,
    pub capabilities_scroll: u16,
    pub actions_scroll: u16,
    pub log_cache: Vec<Event>,
    pub last_log_emit: Instant,
    pub hold_state: Option<HoldState>,
    pub pinned_tooltip: Option<Tooltip>,
    pub search_active: bool,
    pub search_query: String,
    pub show_detail_panel: bool,
    pub hover_node_id: Option<String>,
    pub detail_scroll: u16,
    pub detail_area: Rect,
}

impl UiState {
    /// Construct a default UI state.
    pub fn new() -> Self {
        Self {
            screen_area: Rect::default(),
            actions_area: Rect::default(),
            navbar_item_areas: [Rect::default(); 3],
            nav_flyout_area: Rect::default(),
            nav_flyout_item_areas: [Rect::default(); 12],
            nav_flyout_count: 0,
            assembly_area: Rect::default(),
            body_area: Rect::default(),
            capabilities_area: Rect::default(),
            logs_area: Rect::default(),
            hover_capability_index: None,
            hover_action_index: None,
            hover_panel: HoverPanel::None,
            mouse_pos: None,
            collapsed_capabilities: false,
            collapsed_assembly_steps: false,
            collapsed_actions: false,
            collapsed_logs: false,
            collapsed_help: false,
            collapsed_notifications: true,
            auto_refresh: true,
            log_config: LogStreamConfig::default(),
            log_paused: false,
            log_scroll: 0,
            assembly_scroll: 0,
            capabilities_scroll: 0,
            actions_scroll: 0,
            log_cache: Vec::new(),
            last_log_emit: Instant::now(),
            hold_state: None,
            pinned_tooltip: None,
            search_active: false,
            search_query: String::new(),
            show_detail_panel: false,
            hover_node_id: None,
            detail_scroll: 0,
            detail_area: Rect::default(),
        }
    }
}

impl Default for UiState {
    fn default() -> Self {
        Self::new()
    }
}
