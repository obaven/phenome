use ratatui::layout::Rect;
use std::time::Instant;

use crate::logging::LogStreamConfig;
use crate::runtime::Event;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HoverPanel {
    None,
    Plan,
    Capabilities,
    Actions,
    Logs,
    Problems,
    Help,
    Settings,
}

#[derive(Debug, Clone)]
pub struct Tooltip {
    pub title: String,
    pub lines: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct HoldState {
    pub key: char,
    pub started_at: Instant,
    pub triggered: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogMenuMode {
    Filter,
    Stream,
}

pub struct UiState {
    pub screen_area: Rect,
    pub actions_area: Rect,
    pub settings_area: Rect,
    pub settings_gear_area: Rect,
    pub log_controls_area: Rect,
    pub plan_area: Rect,
    pub plan_progress_area: Rect,
    pub snapshot_area: Rect,
    pub body_area: Rect,
    pub capabilities_area: Rect,
    pub logs_area: Rect,
    pub problems_area: Rect,
    pub help_area: Rect,
    pub log_menu_area: Rect,
    pub log_filter_tag_area: Rect,
    pub log_stream_tag_area: Rect,
    pub hover_plan_index: Option<usize>,
    pub hover_capability_index: Option<usize>,
    pub hover_action_index: Option<usize>,
    pub hover_problem_index: Option<usize>,
    pub log_menu_hover_index: Option<usize>,
    pub hover_snapshot: bool,
    pub hover_panel: HoverPanel,
    pub mouse_pos: Option<(u16, u16)>,
    pub collapsed_plan_progress: bool,
    pub collapsed_snapshot: bool,
    pub collapsed_capabilities: bool,
    pub collapsed_plan_steps: bool,
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
    pub log_menu_mode: Option<LogMenuMode>,
    pub log_menu_pinned: bool,
    pub auto_refresh: bool,
    pub log_config: LogStreamConfig,
    pub log_paused: bool,
    pub log_scroll: u16,
    pub plan_scroll: u16,
    pub capabilities_scroll: u16,
    pub actions_scroll: u16,
    pub problems_scroll: u16,
    pub log_cache: Vec<Event>,
    pub last_log_emit: Instant,
    pub hold_state: Option<HoldState>,
    pub pinned_tooltip: Option<Tooltip>,
    pub action_flash_index: Option<usize>,
    pub action_flash_at: Option<Instant>,
}

impl UiState {
    pub fn new() -> Self {
        Self {
            screen_area: Rect::default(),
            actions_area: Rect::default(),
            settings_area: Rect::default(),
            settings_gear_area: Rect::default(),
            log_controls_area: Rect::default(),
            plan_area: Rect::default(),
            plan_progress_area: Rect::default(),
            snapshot_area: Rect::default(),
            body_area: Rect::default(),
            capabilities_area: Rect::default(),
            logs_area: Rect::default(),
            problems_area: Rect::default(),
            help_area: Rect::default(),
            log_menu_area: Rect::default(),
            log_filter_tag_area: Rect::default(),
            log_stream_tag_area: Rect::default(),
            hover_plan_index: None,
            hover_capability_index: None,
            hover_action_index: None,
            hover_problem_index: None,
            log_menu_hover_index: None,
            hover_snapshot: false,
            hover_panel: HoverPanel::None,
            mouse_pos: None,
            collapsed_plan_progress: false,
            collapsed_snapshot: false,
            collapsed_capabilities: false,
            collapsed_plan_steps: false,
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
            plan_scroll: 0,
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
