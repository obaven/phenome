use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EventLevel {
    Info,
    Warn,
    Error,
}

impl EventLevel {
    pub fn as_str(self) -> &'static str {
        match self {
            EventLevel::Info => "info",
            EventLevel::Warn => "warn",
            EventLevel::Error => "error",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub timestamp_ms: u64,
    pub level: EventLevel,
    pub message: String,
}

impl Event {
    pub fn new(level: EventLevel, message: impl Into<String>) -> Self {
        Self {
            timestamp_ms: now_millis(),
            level,
            message: message.into(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct EventBus {
    max_events: usize,
    events: VecDeque<Event>,
}

impl EventBus {
    pub fn new(max_events: usize) -> Self {
        Self {
            max_events,
            events: VecDeque::new(),
        }
    }

    pub fn push(&mut self, event: Event) {
        self.events.push_back(event);
        while self.events.len() > self.max_events {
            self.events.pop_front();
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &Event> {
        self.events.iter()
    }

    pub fn len(&self) -> usize {
        self.events.len()
    }

    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new(200)
    }
}

fn now_millis() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}
