use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    layout::{Margin, Position, Rect},
    widgets::ListState,
};
use std::time::{Duration, Instant};

use crate::adapters::bootstrappo::BootstrappoBackend;
use crate::logging::LOG_INTERVALS_SECS;
use crate::runtime::{ActionId, ActionSafety, Event, EventLevel, Runtime};
use crate::ui::state::UiState;
use crate::ui::layout::{
    GroupPolicy, PanelPriority, SlotPolicy, SLOT_ACTIONS, SLOT_CAPABILITIES,
    SLOT_FOOTER_HELP, SLOT_FOOTER_SETTINGS, SLOT_LOGS, SLOT_LOG_CONTROLS,
    SLOT_PLAN_PROGRESS, SLOT_PLAN_STEPS, SLOT_PROBLEMS, SLOT_SNAPSHOT,
};

mod actions;
mod collapse;
mod hover;
mod input;
mod scroll;
mod tooltips;

const COLLAPSED_HEIGHT: u16 = 2;
const LOG_CONTROLS_BASE_HEIGHT: u16 = 3;
const LOG_MENU_FILTER_LEN: u16 = 4;
const LOG_MENU_STREAM_LEN: u16 = (LOG_INTERVALS_SECS.len() + 1) as u16;
const FILTER_LABEL: &str = "Filter ";
const STREAM_LABEL: &str = "Stream ";

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum PanelId {
    PlanProgress,
    Snapshot,
    Capabilities,
    PlanSteps,
    Actions,
    Settings,
    LogControls,
    Problems,
    Logs,
    Help,
}

impl PanelId {
    pub fn slot_id(self) -> Option<&'static str> {
        match self {
            PanelId::PlanProgress => Some(SLOT_PLAN_PROGRESS),
            PanelId::Snapshot => Some(SLOT_SNAPSHOT),
            PanelId::Capabilities => Some(SLOT_CAPABILITIES),
            PanelId::PlanSteps => Some(SLOT_PLAN_STEPS),
            PanelId::Actions => Some(SLOT_ACTIONS),
            PanelId::Settings => Some(SLOT_FOOTER_SETTINGS),
            PanelId::LogControls => Some(SLOT_LOG_CONTROLS),
            PanelId::Problems => Some(SLOT_PROBLEMS),
            PanelId::Logs => Some(SLOT_LOGS),
            PanelId::Help => Some(SLOT_FOOTER_HELP),
        }
    }
}

pub struct App {
    pub backend: BootstrappoBackend,
    pub runtime: Runtime,
    pub action_state: ListState,
    pub confirm: Option<ConfirmPrompt>,
    pub last_refresh: Instant,
    pub should_quit: bool,
    pub ui: UiState,
    pub layout_policy: crate::ui::layout::LayoutPolicy,
}

#[derive(Debug, Clone)]
pub struct ConfirmPrompt {
    pub action_id: ActionId,
    pub label: String,
    pub safety: ActionSafety,
}

impl App {
    pub fn new(backend: BootstrappoBackend) -> Self {
        let mut runtime = backend.runtime();
        runtime.events_mut().push(Event::new(
            EventLevel::Info,
            format!(
                "Connected to Bootstrappo ({})",
                backend.config.network.host_domain
            ),
        ));
        runtime.events_mut().push(Event::new(
            EventLevel::Info,
            format!("Plan path: {}", backend.plan_path.display()),
        ));
        if let Some(error) = &backend.plan_error {
            runtime.events_mut().push(Event::new(
                EventLevel::Warn,
                format!("Plan load failed: {}", error),
            ));
        }
        if let Some(live) = &backend.live_status {
            if let Some(error) = live.last_error() {
                runtime.events_mut().push(Event::new(
                    EventLevel::Warn,
                    format!("Live status unavailable: {}", error),
                ));
            }
        }

        let mut action_state = ListState::default();
        if !runtime.registry().actions().is_empty() {
            action_state.select(Some(0));
        }

        let mut app = Self {
            backend,
            runtime,
            action_state,
            confirm: None,
            last_refresh: Instant::now(),
            should_quit: false,
            ui: UiState::new(),
            layout_policy: crate::ui::layout::LayoutPolicy::new(),
        };
        app.configure_layout_policy();
        app.sync_layout_policy();
        app.refresh_log_cache(true);
        app
    }

    fn configure_layout_policy(&mut self) {
        self.layout_policy.set_policy(
            SLOT_PLAN_PROGRESS,
            SlotPolicy::new(PanelPriority::High),
        );
        self.layout_policy
            .set_policy(SLOT_SNAPSHOT, SlotPolicy::new(PanelPriority::High));
        self.layout_policy
            .set_policy(SLOT_CAPABILITIES, SlotPolicy::new(PanelPriority::Normal));
        self.layout_policy
            .set_policy(SLOT_PLAN_STEPS, SlotPolicy::new(PanelPriority::High));
        self.layout_policy
            .set_policy(SLOT_ACTIONS, SlotPolicy::new(PanelPriority::Normal));
        self.layout_policy
            .set_policy(SLOT_PROBLEMS, SlotPolicy::new(PanelPriority::Low));
        self.layout_policy
            .set_policy(SLOT_LOG_CONTROLS, SlotPolicy::new(PanelPriority::Normal));
        self.layout_policy
            .set_policy(SLOT_LOGS, SlotPolicy::new(PanelPriority::Normal));
        self.layout_policy
            .set_policy(SLOT_FOOTER_HELP, SlotPolicy::new(PanelPriority::Low));
        self.layout_policy
            .set_policy(SLOT_FOOTER_SETTINGS, SlotPolicy::new(PanelPriority::Low));

        self.layout_policy.set_group(
            GroupPolicy::new(
                "left_column",
                vec![
                    SLOT_PLAN_PROGRESS.into(),
                    SLOT_SNAPSHOT.into(),
                    SLOT_CAPABILITIES.into(),
                ],
            )
            .min_area(0, 12),
        );
        self.layout_policy.set_group(
            GroupPolicy::new(
                "middle_aux",
                vec![
                    SLOT_PLAN_STEPS.into(),
                    SLOT_FOOTER_HELP.into(),
                    SLOT_LOGS.into(),
                ],
            )
            .min_area(0, 12),
        );
        self.layout_policy.set_group(
            GroupPolicy::new(
                "right_left",
                vec![SLOT_ACTIONS.into(), SLOT_PROBLEMS.into()],
            )
            .min_area(0, 10),
        );
        self.layout_policy.set_group(
            GroupPolicy::new(
                "right_right",
                vec![SLOT_LOG_CONTROLS.into(), SLOT_LOGS.into()],
            )
            .min_area(0, 10),
        );

    }

    pub fn on_tick(&mut self) {
        if self.ui.auto_refresh && self.last_refresh.elapsed() >= Duration::from_secs(1) {
            self.runtime.refresh_snapshot();
            self.last_refresh = Instant::now();
        }
        self.refresh_log_cache(false);

        let hold_trigger = if let Some(hold) = &mut self.ui.hold_state {
            if !hold.triggered && hold.started_at.elapsed() >= Duration::from_secs(3) {
                hold.triggered = true;
                Some(hold.key)
            } else {
                None
            }
        } else {
            None
        };
        if let Some(key) = hold_trigger {
            match key {
                'p' => self.pin_tooltip(),
                'u' => self.unpin_tooltip(),
                _ => {}
            }
        }

        if !self.ui.log_paused
            && self.ui.last_log_emit.elapsed() >= self.ui.log_config.interval
        {
            self.ui.last_log_emit = Instant::now();
        }
    }

    pub fn handle_key_event(&mut self, key: KeyEvent) -> Result<()> {
        if self.confirm.is_some() {
            return self.handle_confirm_key(key);
        }
        if self.handle_hold_key(&key) {
            return Ok(());
        }
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            self.should_quit = true;
            return Ok(());
        }

        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => self.should_quit = true,
            KeyCode::Char('r') => self.runtime.refresh_snapshot(),
            KeyCode::Char('f') => {
                self.ui.log_config.filter = self.ui.log_config.filter.next();
                self.refresh_log_cache(true);
            }
            KeyCode::Char('s') => self.toggle_settings_panel(),
            KeyCode::Char('w') => self.ui.auto_refresh = !self.ui.auto_refresh,
            KeyCode::Char('a') => self.handle_settings_shortcut(true),
            KeyCode::Char('c') => self.handle_settings_shortcut(false),
            KeyCode::Up | KeyCode::Char('k') => self.select_previous_action(),
            KeyCode::Down | KeyCode::Char('j') => self.select_next_action(),
            KeyCode::Enter => self.trigger_selected_action()?,
            _ => {}
        }
        Ok(())
    }

    fn handle_settings_shortcut(&mut self, apply: bool) {
        if self.panel_collapsed(PanelId::Settings) {
            return;
        }
        self.ui.settings_selected = if apply { 0 } else { 1 };
        let message = if apply {
            "Settings: apply (shortcut)"
        } else {
            "Settings: cancel (shortcut)"
        };
        self.runtime
            .events_mut()
            .push(Event::new(EventLevel::Info, message));
    }

    pub fn close_log_menu(&mut self) {
        self.ui.log_menu_pinned = false;
        self.ui.log_menu_mode = None;
        self.ui.log_menu_len = 0;
        self.ui.log_menu_area = Rect::default();
        self.ui.log_menu_hover_index = None;
    }

    pub fn sync_layout_policy(&mut self) {
        let panels = [
            PanelId::PlanProgress,
            PanelId::Snapshot,
            PanelId::Capabilities,
            PanelId::PlanSteps,
            PanelId::Actions,
            PanelId::Settings,
            PanelId::LogControls,
            PanelId::Problems,
            PanelId::Logs,
            PanelId::Help,
        ];
        for panel in panels {
            if let Some(slot) = panel.slot_id() {
                let slot_id = slot.into();
                self.layout_policy.clear_override(&slot_id);
                self.layout_policy
                    .set_collapsed(slot, self.is_collapsed(panel));
            }
        }
    }

    pub fn log_controls_height(&self) -> u16 {
        if self.panel_collapsed(PanelId::LogControls) {
            return COLLAPSED_HEIGHT;
        }
        let menu_items = if self.ui.log_menu_pinned {
            match self.ui.log_menu_mode {
                Some(crate::ui::state::LogMenuMode::Filter) => LOG_MENU_FILTER_LEN,
                Some(crate::ui::state::LogMenuMode::Stream) => LOG_MENU_STREAM_LEN,
                None => 0,
            }
        } else {
            0
        };
        let menu_height = if menu_items > 0 { menu_items + 2 } else { 0 };
        LOG_CONTROLS_BASE_HEIGHT.saturating_add(menu_height)
    }

    pub fn panel_visible(&self, panel: PanelId) -> bool {
        if let Some(slot) = panel.slot_id() {
            if let Some(visible) = self.layout_policy.visibility_for(slot) {
                return visible;
            }
        }
        true
    }

    pub fn panel_collapsed(&self, panel: PanelId) -> bool {
        if let Some(slot) = panel.slot_id() {
            if let Some(collapsed) = self.layout_policy.collapsed_for(slot) {
                return collapsed;
            }
        }
        self.is_collapsed(panel)
    }

    fn log_menu_trigger_contains(&self, pos: Position) -> bool {
        if self.ui.log_filter_tag_area.contains(pos)
            || self.ui.log_stream_tag_area.contains(pos)
        {
            return true;
        }
        if self.ui.log_controls_area.height == 0 {
            return false;
        }
        let inner = self.ui.log_controls_area.inner(Margin {
            horizontal: 1,
            vertical: 1,
        });
        if inner.height == 0 || pos.y != inner.y {
            return false;
        }
        let filter_start =
            self.ui.log_filter_tag_area.x.saturating_sub(FILTER_LABEL.len() as u16);
        let filter_end = self
            .ui
            .log_filter_tag_area
            .x
            .saturating_add(self.ui.log_filter_tag_area.width);
        let stream_start =
            self.ui.log_stream_tag_area.x.saturating_sub(STREAM_LABEL.len() as u16);
        let stream_end = self
            .ui
            .log_stream_tag_area
            .x
            .saturating_add(self.ui.log_stream_tag_area.width);
        let x = pos.x;
        (x >= filter_start && x < filter_end) || (x >= stream_start && x < stream_end)
    }
}
