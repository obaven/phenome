//! Application initialization and periodic tick behavior.

use std::time::{Duration, Instant};

use ratatui::widgets::ListState;

use rotappo_domain::{Event, EventLevel};

use super::{App, AppContext};

impl App {
    /// Create a new application instance from an injected runtime and context.
    pub fn new(mut runtime: rotappo_application::Runtime, context: AppContext) -> Self {
        let host_domain = &context.host_domain;
        let assembly_path = context.assembly_path.display();
        runtime.events_mut().push(Event::new(
            EventLevel::Info,
            format!("Connected to Bootstrappo ({host_domain})"),
        ));
        runtime.events_mut().push(Event::new(
            EventLevel::Info,
            format!("Assembly path: {assembly_path}"),
        ));
        if let Some(error) = &context.assembly_error {
            runtime.events_mut().push(Event::new(
                EventLevel::Warn,
                format!("Assembly load failed: {error}"),
            ));
        }
        if let Some(error) = &context.live_status_error {
            runtime.events_mut().push(Event::new(
                EventLevel::Warn,
                format!("Live status unavailable: {error}"),
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
            graph: crate::app::GraphRenderState::new(),
            active_nav: crate::app::NavSection::Analytics,
            active_view: crate::app::NavView::AnalyticsRealtime,
            nav_sub_index: [0; 3],
            analytics_client: None,
            analytics_metrics: None,
            analytics_anomalies: None,
            analytics_recommendations: None,
            analytics_cache_timestamp: None,
            analytics_rx: None,
        };

        // Spawn analytics background task
        if let Ok(client) = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current()
                .block_on(crate::analytics_client::AnalyticsClient::connect_from_env())
        }) {
            let client = client.clone(); // It is cloneable
            app.analytics_client = Some(client.clone());
            let (tx, rx) = tokio::sync::mpsc::channel(10);
            app.analytics_rx = Some(rx);

            tokio::spawn(async move {
                loop {
                    if let Ok(metrics) = client.fetch_metrics().await {
                        let _ = tx
                            .send(crate::app::core::AnalyticsUpdate::Metrics(metrics))
                            .await;
                    }
                    if let Ok(anomalies) = client.fetch_anomalies().await {
                        let _ = tx
                            .send(crate::app::core::AnalyticsUpdate::Anomalies(anomalies))
                            .await;
                    }
                    if let Ok(recs) = client.fetch_recommendations().await {
                        let _ = tx
                            .send(crate::app::core::AnalyticsUpdate::Recommendations(recs))
                            .await;
                    }
                    tokio::time::sleep(Duration::from_secs(5)).await;
                }
            });
        }

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
        self.refresh_analytics_cache();

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

        if !self.ui.log_paused && self.ui.last_log_emit.elapsed() >= self.ui.log_config.interval {
            self.ui.last_log_emit = Instant::now();
        }
    }

    fn refresh_analytics_cache(&mut self) {
        if let Some(rx) = &mut self.analytics_rx {
            while let Ok(update) = rx.try_recv() {
                match update {
                    crate::app::core::AnalyticsUpdate::Metrics(m) => {
                        self.analytics_metrics = Some(m)
                    }
                    crate::app::core::AnalyticsUpdate::Anomalies(a) => {
                        self.analytics_anomalies = Some(a)
                    }
                    crate::app::core::AnalyticsUpdate::Recommendations(r) => {
                        self.analytics_recommendations = Some(r)
                    }
                }
                self.analytics_cache_timestamp = Some(Instant::now());
            }
        }
    }

    fn configure_layout_policy(&mut self) {
        use crate::layout::{
            GroupPolicy, PanelPriority, SLOT_ACTIONS, SLOT_ASSEMBLY_PROGRESS, SLOT_ASSEMBLY_STEPS,
            SLOT_CAPABILITIES, SLOT_FOOTER_HELP, SLOT_FOOTER_SETTINGS, SLOT_LOG_CONTROLS,
            SLOT_LOGS, SLOT_NOTIFICATIONS, SLOT_PROBLEMS, SLOT_SNAPSHOT, SlotPolicy,
        };

        self.layout_policy
            .set_policy(SLOT_ASSEMBLY_PROGRESS, SlotPolicy::new(PanelPriority::High));
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
            .set_policy(SLOT_NOTIFICATIONS, SlotPolicy::new(PanelPriority::Low));
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
