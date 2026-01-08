//! Application initialization and periodic tick behavior.

use std::time::{Duration, Instant};

use ratatui::widgets::ListState;

use rotappo_domain::{Event, EventLevel};

use super::{App, AppContext};

impl App {
    /// Create a new application instance from an injected runtime and context.
    pub fn new(mut runtime: rotappo_application::Runtime, context: AppContext) -> Self {
        runtime.events_mut().push(Event::new(
            EventLevel::Info,
            format!("Connected to Bootstrappo ({})", context.host_domain),
        ));
        runtime.events_mut().push(Event::new(
            EventLevel::Info,
            format!("Assembly path: {}", context.assembly_path.display()),
        ));
        if let Some(error) = &context.assembly_error {
            runtime.events_mut().push(Event::new(
                EventLevel::Warn,
                format!("Assembly load failed: {}", error),
            ));
        }
        if let Some(error) = &context.live_status_error {
            runtime.events_mut().push(Event::new(
                EventLevel::Warn,
                format!("Live status unavailable: {}", error),
            ));
        }

        let mut action_state = ListState::default();
        if !runtime.registry().actions().is_empty() {
            action_state.select(Some(0));
        }

        let mut app = Self {
            runtime,
            context,
            action_state,
            confirm: None,
            last_refresh: Instant::now(),
            should_quit: false,
            ui: crate::state::UiState::new(),
            layout_policy: crate::layout::LayoutPolicy::new(),
        };
        app.configure_layout_policy();
        app.sync_layout_policy();
        app.refresh_log_cache(true);
        app
    }

    /// Update time-sensitive state and log caches.
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

    fn configure_layout_policy(&mut self) {
        use crate::layout::{
            GroupPolicy, PanelPriority, SlotPolicy, SLOT_ACTIONS, SLOT_CAPABILITIES,
            SLOT_FOOTER_HELP, SLOT_FOOTER_SETTINGS, SLOT_LOGS, SLOT_LOG_CONTROLS,
            SLOT_ASSEMBLY_PROGRESS, SLOT_ASSEMBLY_STEPS, SLOT_PROBLEMS, SLOT_SNAPSHOT,
        };

        self.layout_policy.set_policy(
            SLOT_ASSEMBLY_PROGRESS,
            SlotPolicy::new(PanelPriority::High),
        );
        self.layout_policy
            .set_policy(SLOT_SNAPSHOT, SlotPolicy::new(PanelPriority::High));
        self.layout_policy
            .set_policy(SLOT_CAPABILITIES, SlotPolicy::new(PanelPriority::Normal));
        self.layout_policy
            .set_policy(SLOT_ASSEMBLY_STEPS, SlotPolicy::new(PanelPriority::High));
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
                    SLOT_ASSEMBLY_PROGRESS.into(),
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
                    SLOT_ASSEMBLY_STEPS.into(),
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
}
