use primer::application::events::InteractiveCommand;

#[derive(Default)]
pub struct MenuState {
    pub active: bool,
    pub selected: usize,
    pub confirming: bool,
    pub confirmation_message: String,
    pub pending_command: Option<InteractiveCommand>,
    pub timeout_input: Option<String>,
}

impl MenuState {
    pub fn open(&mut self) {
        self.active = true;
        self.selected = 0;
        self.confirming = false;
        self.timeout_input = None;
    }

    pub fn confirm(&mut self, message: String, cmd: InteractiveCommand) {
        self.confirming = true;
        self.confirmation_message = message;
        self.pending_command = Some(cmd);
    }

    pub fn clear(&mut self) {
        self.active = false;
        self.selected = 0;
        self.confirming = false;
        self.confirmation_message.clear();
        self.pending_command = None;
        self.timeout_input = None;
    }
}

#[derive(Clone, Copy)]
pub enum MenuAction {
    Skip,
    Retry,
    AdjustTimeout,
    ViewLogs,
    ToggleExpand,
    Pause,
    Resume,
    Cancel,
}

impl MenuAction {
    pub fn label(self) -> &'static str {
        match self {
            MenuAction::Skip => "Skip Component",
            MenuAction::Retry => "Retry Component",
            MenuAction::AdjustTimeout => "Adjust Timeout",
            MenuAction::ViewLogs => "View Logs",
            MenuAction::ToggleExpand => "Expand Details",
            MenuAction::Pause => "Pause Bootstrap",
            MenuAction::Resume => "Resume Bootstrap",
            MenuAction::Cancel => "Cancel Bootstrap",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            MenuAction::Skip => "Mark as deferred and continue",
            MenuAction::Retry => "Retry selected component",
            MenuAction::AdjustTimeout => "Change readiness timeout",
            MenuAction::ViewLogs => "Show recent bootstrap logs",
            MenuAction::ToggleExpand => "Show readiness details",
            MenuAction::Pause => "Pause component execution",
            MenuAction::Resume => "Resume component execution",
            MenuAction::Cancel => "Stop bootstrap",
        }
    }
}
