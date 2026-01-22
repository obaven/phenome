use super::super::{NavAction, NavSubItem, NavView};

pub(super) const ANALYTICS_ITEMS: [NavSubItem; 6] = [
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
