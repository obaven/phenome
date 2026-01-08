use std::time::Duration;

use rotappo_domain::EventLevel;

pub const LOG_INTERVALS_SECS: [u64; 4] = [1, 2, 5, 10];
pub const DEFAULT_LOG_INTERVAL_SECS: u64 = 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogFilter {
    All,
    Info,
    Warn,
    Error,
}

impl LogFilter {
    pub fn next(self) -> Self {
        match self {
            LogFilter::All => LogFilter::Info,
            LogFilter::Info => LogFilter::Warn,
            LogFilter::Warn => LogFilter::Error,
            LogFilter::Error => LogFilter::All,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            LogFilter::All => "all",
            LogFilter::Info => "info",
            LogFilter::Warn => "warn",
            LogFilter::Error => "error",
        }
    }

    pub fn matches(self, level: EventLevel) -> bool {
        match self {
            LogFilter::All => true,
            LogFilter::Info => level == EventLevel::Info,
            LogFilter::Warn => level == EventLevel::Warn,
            LogFilter::Error => level == EventLevel::Error,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct LogStreamConfig {
    pub interval: Duration,
    pub filter: LogFilter,
}

impl Default for LogStreamConfig {
    fn default() -> Self {
        Self {
            interval: Duration::from_secs(DEFAULT_LOG_INTERVAL_SECS),
            filter: LogFilter::All,
        }
    }
}

pub fn next_log_interval_secs(current: u64) -> u64 {
    for (idx, value) in LOG_INTERVALS_SECS.iter().enumerate() {
        if *value == current {
            return LOG_INTERVALS_SECS[(idx + 1) % LOG_INTERVALS_SECS.len()];
        }
    }
    LOG_INTERVALS_SECS[0]
}
