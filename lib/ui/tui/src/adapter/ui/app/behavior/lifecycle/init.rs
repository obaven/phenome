use ratatui::widgets::ListState;
use std::time::Instant;

use phenome_domain::{Event, EventLevel};

use crate::app::{App, AppContext};

impl App {
    pub fn new(mut runtime: phenome_application::Runtime, context: AppContext) -> Self {
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

        app.start_analytics();
        app.refresh_log_cache(true);
        app
    }
}
