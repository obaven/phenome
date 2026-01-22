use super::super::{NavAction, NavSubItem, NavView};

pub(super) const TERMINAL_ITEMS: [NavSubItem; 7] = [
    NavSubItem {
        label: "Log Stream",
        view: NavView::TerminalLogs,
        action: NavAction::None,
    },
    NavSubItem {
        label: "Event Feed",
        view: NavView::TerminalEvents,
        action: NavAction::None,
    },
    NavSubItem {
        label: "Commands",
        view: NavView::TerminalCommands,
        action: NavAction::None,
    },
    NavSubItem {
        label: "Diagnostics",
        view: NavView::TerminalDiagnostics,
        action: NavAction::ToggleNotifications,
    },
    NavSubItem {
        label: "Toggle Watch",
        view: NavView::TerminalLogs,
        action: NavAction::ToggleWatch,
    },
    NavSubItem {
        label: "Cycle Filter",
        view: NavView::TerminalLogs,
        action: NavAction::CycleLogFilter,
    },
    NavSubItem {
        label: "Next Interval",
        view: NavView::TerminalLogs,
        action: NavAction::NextLogInterval,
    },
];
