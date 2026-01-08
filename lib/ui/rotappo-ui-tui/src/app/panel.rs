//! Panel identifiers used for layout, hover, and collapse logic.

use crate::layout::{
    SLOT_ACTIONS, SLOT_CAPABILITIES, SLOT_FOOTER_HELP, SLOT_FOOTER_SETTINGS, SLOT_LOGS,
    SLOT_LOG_CONTROLS, SLOT_ASSEMBLY_PROGRESS, SLOT_ASSEMBLY_STEPS, SLOT_PROBLEMS, SLOT_SNAPSHOT,
};

/// Panels displayed in the TUI shell.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PanelId {
    AssemblyProgress,
    Snapshot,
    Capabilities,
    AssemblySteps,
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
            PanelId::AssemblyProgress => Some(SLOT_ASSEMBLY_PROGRESS),
            PanelId::Snapshot => Some(SLOT_SNAPSHOT),
            PanelId::Capabilities => Some(SLOT_CAPABILITIES),
            PanelId::AssemblySteps => Some(SLOT_ASSEMBLY_STEPS),
            PanelId::Actions => Some(SLOT_ACTIONS),
            PanelId::Settings => Some(SLOT_FOOTER_SETTINGS),
            PanelId::LogControls => Some(SLOT_LOG_CONTROLS),
            PanelId::Problems => Some(SLOT_PROBLEMS),
            PanelId::Logs => Some(SLOT_LOGS),
            PanelId::Help => Some(SLOT_FOOTER_HELP),
        }
    }
}
