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
