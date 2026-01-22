use phenome_ui_presentation::logging::next_log_interval_secs;

use crate::app::App;

impl App {
    pub fn cycle_log_interval(&mut self) {
        let current = self.ui.log_config.interval.as_secs();
        let next = next_log_interval_secs(current);
        self.ui.log_config.interval = std::time::Duration::from_secs(next);
    }

    pub fn filtered_events(&self) -> Vec<&phenome_domain::Event> {
        self.ui.log_cache.iter().collect()
    }

    pub fn refresh_log_cache(&mut self, force: bool) {
        if !force {
            if self.ui.log_paused {
                return;
            }
            if self.ui.last_log_emit.elapsed() < self.ui.log_config.interval {
                return;
            }
        }
        self.ui.last_log_emit = std::time::Instant::now();
        self.ui.log_cache = self
            .runtime
            .events()
            .iter()
            .filter(|event| self.ui.log_config.filter.matches(event.level))
            .cloned()
            .collect();
    }
}
