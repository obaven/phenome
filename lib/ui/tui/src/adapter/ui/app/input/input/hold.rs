use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};

use crate::state::HoldState;

use crate::app::App;

impl App {
    pub fn handle_hold_key(&mut self, key: &KeyEvent) -> bool {
        let pressed = key.kind == KeyEventKind::Press;
        let released = key.kind == KeyEventKind::Release;
        let KeyCode::Char(ch) = key.code else {
            return false;
        };
        if !matches!(ch, 'p' | 'u') {
            return false;
        }
        if pressed {
            self.start_hold(ch);
            return true;
        }
        if released {
            self.finish_hold(ch);
            return true;
        }
        false
    }

    pub fn start_hold(&mut self, key: char) {
        self.ui.hold_state = Some(HoldState {
            key,
            started_at: std::time::Instant::now(),
            triggered: false,
        });
    }

    pub fn finish_hold(&mut self, key: char) {
        if let Some(hold) = &self.ui.hold_state {
            if hold.key == key {
                if !hold.triggered && key == 'p' {
                    self.ui.log_paused = !self.ui.log_paused;
                }
                self.ui.hold_state = None;
            }
        }
    }
}
