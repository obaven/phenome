//! Navbar state and navigation helpers.

use super::App;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NavSection {
    Analytics,
    Topology,
    Terminal,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NavView {
    AnalyticsRealtime,
    AnalyticsHistorical,
    AnalyticsPredictions,
    AnalyticsRecommendations,
    AnalyticsInsights,
    TopologyAssembly,
    TopologyDomains,
    TopologyCapabilities,
    TopologyQueue,
    TopologyHealth,
    TopologyDagGraph,
    TopologyDualGraph,
    TerminalLogs,
    TerminalEvents,
    TerminalCommands,
    TerminalDiagnostics,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NavAction {
    None,
    RefreshSnapshot,
    ToggleNotifications,
    ToggleWatch,
    CycleLogFilter,
    NextLogInterval,
}

#[derive(Clone, Copy, Debug)]
pub struct NavSubItem {
    pub label: &'static str,
    pub view: NavView,
    pub action: NavAction,
}

impl NavSection {
    pub const ALL: [NavSection; 3] = [
        NavSection::Analytics,
        NavSection::Topology,
        NavSection::Terminal,
    ];

    pub fn index(self) -> usize {
        match self {
            NavSection::Analytics => 0,
            NavSection::Topology => 1,
            NavSection::Terminal => 2,
        }
    }

    pub fn from_index(index: usize) -> Self {
        match index {
            1 => NavSection::Topology,
            2 => NavSection::Terminal,
            _ => NavSection::Analytics,
        }
    }

    pub fn title(self) -> &'static str {
        match self {
            NavSection::Analytics => "Analytics",
            NavSection::Topology => "Topology",
            NavSection::Terminal => "Terminal",
        }
    }

    pub fn next(self) -> Self {
        Self::from_index((self.index() + 1) % Self::ALL.len())
    }

    pub fn prev(self) -> Self {
        let len = Self::ALL.len();
        Self::from_index((self.index() + len - 1) % len)
    }
}

const ANALYTICS_ITEMS: [NavSubItem; 6] = [
    NavSubItem {
        label: "Real-time",
        view: NavView::AnalyticsRealtime,
        action: NavAction::None,
    },
    NavSubItem {
        label: "Historical",
        view: NavView::AnalyticsHistorical,
        action: NavAction::None,
    },
    NavSubItem {
        label: "Predictions",
        view: NavView::AnalyticsPredictions,
        action: NavAction::None,
    },
    NavSubItem {
        label: "Recommendations",
        view: NavView::AnalyticsRecommendations,
        action: NavAction::None,
    },
    NavSubItem {
        label: "Insights",
        view: NavView::AnalyticsInsights,
        action: NavAction::None,
    },
    NavSubItem {
        label: "Refresh Snapshot",
        view: NavView::AnalyticsRealtime,
        action: NavAction::RefreshSnapshot,
    },
];

const TOPOLOGY_ITEMS: [NavSubItem; 8] = [
    NavSubItem {
        label: "Assembly Steps",
        view: NavView::TopologyAssembly,
        action: NavAction::None,
    },
    NavSubItem {
        label: "Domains",
        view: NavView::TopologyDomains,
        action: NavAction::None,
    },
    NavSubItem {
        label: "Capabilities",
        view: NavView::TopologyCapabilities,
        action: NavAction::None,
    },
    NavSubItem {
        label: "Queue State",
        view: NavView::TopologyQueue,
        action: NavAction::None,
    },
    NavSubItem {
        label: "Health",
        view: NavView::TopologyHealth,
        action: NavAction::None,
    },
    NavSubItem {
        label: "DAG Graph",
        view: NavView::TopologyDagGraph,
        action: NavAction::None,
    },
    NavSubItem {
        label: "Dual Graph",
        view: NavView::TopologyDualGraph,
        action: NavAction::None,
    },
    NavSubItem {
        label: "Refresh Snapshot",
        view: NavView::TopologyAssembly,
        action: NavAction::RefreshSnapshot,
    },
];

const TERMINAL_ITEMS: [NavSubItem; 7] = [
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

pub fn nav_items(section: NavSection) -> &'static [NavSubItem] {
    match section {
        NavSection::Analytics => &ANALYTICS_ITEMS,
        NavSection::Topology => &TOPOLOGY_ITEMS,
        NavSection::Terminal => &TERMINAL_ITEMS,
    }
}

impl App {
    pub fn active_nav(&self) -> NavSection {
        self.active_nav
    }

    pub fn active_view(&self) -> NavView {
        self.active_view
    }

    pub fn set_active_nav(&mut self, nav: NavSection) {
        self.active_nav = nav;
        let items = nav_items(nav);
        if items.is_empty() {
            return;
        }
        let index = self.nav_sub_index[nav.index()].min(items.len().saturating_sub(1));
        self.nav_sub_index[nav.index()] = index;
        self.active_view = items[index].view;
    }

    pub fn next_nav(&mut self) {
        self.active_nav = self.active_nav.next();
        self.set_active_nav(self.active_nav);
    }

    pub fn prev_nav(&mut self) {
        self.active_nav = self.active_nav.prev();
        self.set_active_nav(self.active_nav);
    }

    pub fn nav_sub_index(&self, section: NavSection) -> usize {
        self.nav_sub_index[section.index()]
    }

    pub fn set_nav_sub_index(&mut self, index: usize) {
        let section = self.active_nav;
        let items = nav_items(section);
        if items.is_empty() {
            return;
        }
        let clamped = index.min(items.len().saturating_sub(1));
        self.nav_sub_index[section.index()] = clamped;
        self.active_view = items[clamped].view;
    }

    pub fn activate_nav_sub(&mut self, index: usize) {
        self.set_nav_sub_index(index);
        let section = self.active_nav;
        let items = nav_items(section);
        if let Some(item) = items.get(self.nav_sub_index(section)) {
            self.execute_nav_action(item.action);
        }
    }

    pub fn next_nav_sub(&mut self) {
        let section = self.active_nav;
        let items = nav_items(section);
        if items.is_empty() {
            return;
        }
        let current = self.nav_sub_index(section);
        let next = (current + 1) % items.len();
        self.set_nav_sub_index(next);
    }

    pub fn prev_nav_sub(&mut self) {
        let section = self.active_nav;
        let items = nav_items(section);
        if items.is_empty() {
            return;
        }
        let current = self.nav_sub_index(section);
        let next = (current + items.len() - 1) % items.len();
        self.set_nav_sub_index(next);
    }

    pub fn active_subitem(&self) -> Option<NavSubItem> {
        let items = nav_items(self.active_nav);
        items.get(self.nav_sub_index(self.active_nav)).copied()
    }

    fn execute_nav_action(&mut self, action: NavAction) {
        match action {
            NavAction::None => {}
            NavAction::RefreshSnapshot => {
                self.runtime.refresh_snapshot();
                self.refresh_log_cache(true);
            }
            NavAction::ToggleNotifications => {
                self.toggle_notifications_panel();
            }
            NavAction::ToggleWatch => {
                self.ui.auto_refresh = !self.ui.auto_refresh;
            }
            NavAction::CycleLogFilter => {
                self.ui.log_config.filter = self.ui.log_config.filter.next();
                self.refresh_log_cache(true);
            }
            NavAction::NextLogInterval => {
                self.cycle_log_interval();
                self.refresh_log_cache(true);
            }
        }
    }
}
