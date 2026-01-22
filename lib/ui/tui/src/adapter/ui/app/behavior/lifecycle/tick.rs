use std::time::{Duration, Instant};

use crate::app::App;

impl App {
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
}
