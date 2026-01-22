use super::super::{NavAction, NavSubItem, NavView};

pub(super) const TOPOLOGY_ITEMS: [NavSubItem; 8] = [
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
